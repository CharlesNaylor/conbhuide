/*
 * Celtic knot around an arbitrary planar graph (Mercat's construction).
 *
 * Every active edge carries one strand crossing at its midpoint M. Around every node, each
 * pair of neighbouring active edges (in angular order) bounds a "corner", and one piece of
 * strand runs through each corner from the midpoint of its first edge to the midpoint of its
 * second. A node with a single edge has one corner that goes all the way round it.
 *
 * Geometry. A corner curve is written in polar coordinates r(theta) around its node:
 *   - it leaves M at angle BETA to the edge, falls to a radius rho, runs round the node at
 *     that radius, and climbs back out to the next midpoint, again at angle BETA;
 *   - BETA = 60 degrees, so that between two edges 60 degrees apart (neighbours in the
 *     triangular mesh) the curve is a straight line, and a fully active mesh is a plain
 *     three-way weave;
 *   - wider corners are pulled in closer to the node, so loops around neighbouring nodes
 *     stay clear of each other.
 * Both pieces of a strand meet M at the same angle, so strands run straight through
 * crossings.
 *
 * Over/under. Each corner curve starts on one strand of its first crossing and ends on the
 * other strand of its second. Drawing every curve's start on top therefore makes every
 * strand alternate over, under, over... with no bookkeeping.
 *
 * Animation. The corner curves round a node join into one closed curve r(theta), which
 * peaks at each active edge's midpoint. To move from one state to the next, r is
 * interpolated between the two closed curves. Where an edge switches on, the curves round
 * its two end nodes each push a tip out towards the midpoint; the tips meet there at the
 * end of the move, at exactly the angle of the crossing that replaces them. A node that
 * gains its first edge grows its curve out of the node itself.
 */
use crate::knot::KnotStyle;
use crate::network::Network;
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_3, PI, TAU};

/// angle between a strand and the edge it crosses
const BETA: f32 = FRAC_PI_3;
/// radius of the strand round a wide corner, as a fraction of the half edge length
const RHO_WIDE: f32 = 0.62;
/// extra corner angle (beyond the straight-line case) over which rho shrinks to RHO_WIDE
const RHO_BLEND: f32 = TAU / 3.0;
/// pieces of strand overlap their neighbours by this many pixels to hide seams
const OVERLAP: f32 = 0.5;

pub struct Curve {
    pub pts: Vec<Vec2>,
    pub normals: Vec<Vec2>,
}

fn perp(v: Vec2) -> Vec2 {
    vec2(-v.y, v.x)
}

fn polar(theta: f32) -> (Vec2, Vec2) {
    let radial = vec2(theta.cos(), theta.sin());
    (radial, perp(radial))
}

/// Radius of a corner curve as it comes in from an edge midpoint: `t` runs from 0 (at the
/// midpoint, radius `a`, leaving at BETA to the edge) to 1 (radius `rho`, level), over an
/// angle of `span` radians.
fn approach_radius(a: f32, rho: f32, span: f32, t: f32) -> f32 {
    let slope = -a / BETA.tan() * span;
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * a + (t3 - 2.0 * t2 + t) * slope + (-2.0 * t3 + 3.0 * t2) * rho
}

impl Curve {
    /// Ribbon centre line through `pts`. The tangent at either end can be given when it is
    /// known exactly. Both ends are extended slightly so adjoining pieces overlap.
    pub fn through(mut pts: Vec<Vec2>, start_tangent: Option<Vec2>, end_tangent: Option<Vec2>) -> Curve {
        let last = pts.len() - 1;
        let mut normals: Vec<Vec2> = (0..pts.len())
            .map(|k| {
                let before = pts[k.saturating_sub(1)];
                let after = pts[(k + 1).min(last)];
                perp((after - before).normalize_or_zero())
            })
            .collect();
        let start_tangent = start_tangent.unwrap_or((pts[1] - pts[0]).normalize_or_zero());
        let end_tangent = end_tangent.unwrap_or((pts[last] - pts[last - 1]).normalize_or_zero());
        normals[0] = perp(start_tangent);
        normals[last] = perp(end_tangent);
        pts.insert(0, pts[0] - start_tangent * OVERLAP);
        normals.insert(0, normals[0]);
        pts.push(pts[last + 1] + end_tangent * OVERLAP);
        normals.push(normals[last + 1]);
        Curve { pts, normals }
    }
}

/// Direction of a strand as it leaves the midpoint of the edge at angle `theta`, heading in
/// towards the node and round it in the direction of increasing angle.
fn leaving_tangent(theta: f32) -> Vec2 {
    let (radial, around) = polar(theta);
    -radial * BETA.cos() + around * BETA.sin()
}

/// Direction of a strand as it arrives at the midpoint of the edge at angle `theta`.
fn arriving_tangent(theta: f32) -> Vec2 {
    let (radial, around) = polar(theta);
    radial * BETA.cos() + around * BETA.sin()
}

/// (angle, radius) points of the strand through a corner that starts on the edge at angle
/// `theta` (half length `a_start`) and sweeps `phi` radians to the next edge (half length
/// `a_end`).
fn corner_polar(theta: f32, phi: f32, a_start: f32, a_end: f32) -> Vec<(f32, f32)> {
    const APPROACH_STEPS: usize = 10;
    let straight_span = PI / 2.0 - BETA;
    let a_mean = (a_start + a_end) / 2.0;
    let blend = ((phi - 2.0 * straight_span) / RHO_BLEND).clamp(0.0, 1.0);
    let blend = blend * blend * (3.0 - 2.0 * blend);
    let rho = a_mean * (BETA.sin() + (RHO_WIDE - BETA.sin()) * blend);
    // angle each end takes to settle to rho: longer when it has further to fall
    let span = |a: f32| (2.0 * (a - rho) * BETA.tan() / a).max(straight_span).min(phi / 2.0);
    let (span_start, span_end) = (span(a_start), span(a_end));
    let level = phi - span_start - span_end;
    let level_steps = (level / 0.15).ceil() as usize;

    let mut polar_pts: Vec<(f32, f32)> = vec![];
    for k in 0..=APPROACH_STEPS {
        let t = k as f32 / APPROACH_STEPS as f32;
        polar_pts.push((theta + t * span_start, approach_radius(a_start, rho, span_start, t)));
    }
    for k in 1..level_steps {
        polar_pts.push((theta + span_start + level * k as f32 / level_steps as f32, rho));
    }
    for k in (0..=APPROACH_STEPS).rev() {
        if k == APPROACH_STEPS && level_steps == 0 {
            continue; // the two approaches meet: this point is already there
        }
        let t = k as f32 / APPROACH_STEPS as f32;
        polar_pts.push((theta + phi - t * span_end, approach_radius(a_end, rho, span_end, t)));
    }
    polar_pts
}

/// The strand through the corner at `center` that starts on the edge at angle `theta` (half
/// length `a_start`) and sweeps `phi` radians to the next edge (half length `a_end`).
pub fn corner_curve(center: Vec2, theta: f32, phi: f32, a_start: f32, a_end: f32) -> Curve {
    let pts = corner_polar(theta, phi, a_start, a_end)
        .iter()
        .map(|&(angle, r)| center + polar(angle).0 * r)
        .collect();
    Curve::through(pts, Some(leaving_tangent(theta)), Some(arriving_tangent(theta + phi)))
}

/// (angle, half length) of each edge at `node` that is on in `active`, in angular order.
fn spokes(net: &Network, node: usize, active: &[bool]) -> Vec<(f32, f32)> {
    let center = net.nodes[node];
    let mut spokes: Vec<(f32, f32)> = net.incident[node]
        .iter()
        .filter(|e| active[**e])
        .map(|e| {
            let d = net.nodes[net.other_end(*e, node)] - center;
            (d.y.atan2(d.x), d.length() / 2.0)
        })
        .collect();
    spokes.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
    spokes
}

/// The corners between `spokes`: (start angle, sweep, start half length, end half length).
fn corners(spokes: &[(f32, f32)]) -> Vec<(f32, f32, f32, f32)> {
    (0..spokes.len())
        .map(|k| {
            let (theta, a_start) = spokes[k];
            let (next_theta, a_end) = spokes[(k + 1) % spokes.len()];
            let phi = if spokes.len() == 1 { TAU } else { (next_theta - theta).rem_euclid(TAU) };
            (theta, phi, a_start, a_end)
        })
        .collect()
}

/// One curve per corner of the active graph.
pub fn corner_curves(net: &Network) -> Vec<Curve> {
    let mut curves = vec![];
    for node in 0..net.nodes.len() {
        for (theta, phi, a_start, a_end) in corners(&spokes(net, node, &net.active)) {
            curves.push(corner_curve(net.nodes[node], theta, phi, a_start, a_end));
        }
    }
    curves
}

pub fn draw_ribbon(curve: &Curve, points: usize, width: f32, color: Color) {
    let half = width / 2.0;
    for k in 1..points.min(curve.pts.len()) {
        let (p, q) = (curve.pts[k - 1], curve.pts[k]);
        let (np, nq) = (curve.normals[k - 1] * half, curve.normals[k] * half);
        draw_triangle(p - np, p + np, q + nq, color);
        draw_triangle(p - np, q + nq, q - nq, color);
    }
}

/// How many leading points of `curve` are needed to cover `length` pixels of it.
fn points_within(curve: &Curve, length: f32) -> usize {
    let mut travelled = 0.0;
    for k in 1..curve.pts.len() {
        travelled += curve.pts[k].distance(curve.pts[k - 1]);
        if travelled >= length {
            return k + 1;
        }
    }
    curve.pts.len()
}

/// A piece of strand ready to draw.
struct Piece {
    curve: Curve,
    width: f32,
    /// How much of the gap to cut in the strand that the start of this piece passes over:
    /// 1 at a crossing, 0 where there is none, in between while one forms or comes apart.
    over: f32,
}

/// Draw `pieces`, and a disc of the given diameter at each of `tips` (the rounded ends of
/// strands that are on their way to or from a crossing).
fn draw_pieces(pieces: &[Piece], tips: &[(Vec2, f32)], gap: f32, style: &KnotStyle) {
    for piece in pieces {
        draw_ribbon(&piece.curve, piece.curve.pts.len(), piece.width, style.strand_color);
    }
    for &(at, diameter) in tips {
        draw_circle(at.x, at.y, diameter / 2.0, style.strand_color);
    }
    // The start of a curve passes over the other strand at its crossing: cut that strand
    // back, then lay the start of the curve over the cut. The strands cross at 2 * BETA.
    let crossing = 2.0 * BETA;
    let covers: Vec<(&Piece, usize)> = pieces
        .iter()
        .filter(|piece| piece.over > 0.0)
        .map(|piece| {
            let half = piece.width / 2.0;
            let cover = half / crossing.sin() + (half + gap * piece.over) / crossing.tan().abs() + 1.5;
            (piece, points_within(&piece.curve, cover))
        })
        .collect();
    for (piece, points) in &covers {
        let cut = piece.width + 2.0 * gap * piece.over;
        draw_ribbon(&piece.curve, *points, cut, style.background);
    }
    for (piece, points) in &covers {
        draw_ribbon(&piece.curve, *points, piece.width, style.strand_color);
    }
}

pub fn draw_network_knot(net: &Network, style: &KnotStyle) {
    let width = net.spacing * style.strand_width;
    let pieces: Vec<Piece> = corner_curves(net)
        .into_iter()
        .map(|curve| Piece { curve, width, over: 1.0 })
        .collect();
    draw_pieces(&pieces, &[], net.spacing * style.gap, style);
}

/// The closed curve round a node as (angle, radius) points, with the angles running up from
/// the first spoke's through one full turn. Empty if the node has no spokes.
fn profile(spokes: &[(f32, f32)]) -> Vec<(f32, f32)> {
    corners(spokes)
        .into_iter()
        .flat_map(|(theta, phi, a_start, a_end)| corner_polar(theta, phi, a_start, a_end))
        .collect()
}

/// Radius of `profile` at `angle`; zero for an empty profile.
fn radius_at(profile: &[(f32, f32)], angle: f32) -> f32 {
    let (first, last) = match (profile.first(), profile.last()) {
        (Some(first), Some(last)) => (first, last),
        _ => return 0.0,
    };
    let angle = first.0 + (angle - first.0).rem_euclid(TAU);
    let after = profile.partition_point(|p| p.0 <= angle);
    if after == 0 {
        return first.1;
    }
    if after == profile.len() {
        return last.1;
    }
    let (p, q) = (profile[after - 1], profile[after]);
    if q.0 - p.0 < 1e-6 {
        return q.1;
    }
    p.1 + (q.1 - p.1) * (angle - p.0) / (q.0 - p.0)
}

/// The over/under gap opens over the last part of a move that ends in a crossing (and closes
/// over the first part of one that starts from a crossing), while the two tips are all but
/// touching. `closeness` is 1 at the crossing and 0 at the other end of the move.
pub fn gap_near_crossing(closeness: f32) -> f32 {
    ((closeness - 0.75) / 0.25).clamp(0.0, 1.0)
}

/// The knot part of the way from the state `previous` (at `t` = 0) to the network's current
/// state (at `t` = 1).
pub fn draw_network_knot_morph(net: &Network, previous: &[bool], t: f32, style: &KnotStyle) {
    /// angle between the points of a moving curve
    const STEP: f32 = 0.06;
    let full_width = net.spacing * style.strand_width;
    let mut pieces: Vec<Piece> = vec![];
    let mut tips: Vec<(Vec2, f32)> = vec![];
    for node in 0..net.nodes.len() {
        let center = net.nodes[node];
        let unchanged = net.incident[node].iter().all(|e| previous[*e] == net.active[*e]);
        if unchanged {
            for (theta, phi, a_start, a_end) in corners(&spokes(net, node, &net.active)) {
                let curve = corner_curve(center, theta, phi, a_start, a_end);
                pieces.push(Piece { curve, width: full_width, over: 1.0 });
            }
            continue;
        }
        let from = profile(&spokes(net, node, previous));
        let to = profile(&spokes(net, node, &net.active));
        let radius = |angle: f32| {
            let (r0, r1) = (radius_at(&from, angle), radius_at(&to, angle));
            r0 + (r1 - r0) * t
        };
        // a curve with nothing to come from or go to is also drawn thinner
        let width = full_width
            * match (from.is_empty(), to.is_empty()) {
                (true, _) => t,
                (_, true) => 1.0 - t,
                _ => 1.0,
            };
        // every edge that is on before or after: (angle, on throughout, gap at its crossing)
        let mut marks: Vec<(f32, bool, f32)> = net.incident[node]
            .iter()
            .filter(|e| previous[**e] || net.active[**e])
            .map(|e| {
                let d = net.nodes[net.other_end(*e, node)] - center;
                let over = match (previous[*e], net.active[*e]) {
                    (true, true) => 1.0,
                    (true, false) => gap_near_crossing(1.0 - t),
                    _ => gap_near_crossing(t),
                };
                (d.y.atan2(d.x), previous[*e] && net.active[*e], over)
            })
            .collect();
        marks.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
        for k in 0..marks.len() {
            let (theta, crossing, over) = marks[k];
            let (next_theta, next_crossing, _) = marks[(k + 1) % marks.len()];
            let phi = if marks.len() == 1 { TAU } else { (next_theta - theta).rem_euclid(TAU) };
            let steps = ((phi / STEP).ceil() as usize).max(2);
            let pts = (0..=steps)
                .map(|i| {
                    let angle = theta + phi * i as f32 / steps as f32;
                    center + polar(angle).0 * radius(angle)
                })
                .collect();
            let curve = Curve::through(
                pts,
                crossing.then(|| leaving_tangent(theta)),
                next_crossing.then(|| arriving_tangent(theta + phi)),
            );
            pieces.push(Piece { curve, width, over });
            if !crossing {
                tips.push((center + polar(theta).0 * radius(theta), width));
            }
        }
    }
    draw_pieces(&pieces, &tips, net.spacing * style.gap, style);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2, tolerance: f32) -> bool {
        a.distance(b) < tolerance
    }

    #[test]
    fn curve_runs_between_the_two_edge_midpoints() {
        let center = vec2(100.0, 100.0);
        for phi in [FRAC_PI_3, 2.0 * FRAC_PI_3, PI, 4.0 * FRAC_PI_3, 5.0 * FRAC_PI_3, TAU] {
            let c = corner_curve(center, 0.3, phi, 20.0, 20.0);
            // index 0 and the last index are the overlap extensions
            assert!(close(c.pts[1], center + polar(0.3).0 * 20.0, 1e-3));
            let end = c.pts[c.pts.len() - 2];
            assert!(close(end, center + polar(0.3 + phi).0 * 20.0, 1e-2), "phi {}", phi);
        }
    }

    #[test]
    fn sixty_degree_corner_is_nearly_straight() {
        let center = vec2(0.0, 0.0);
        let c = corner_curve(center, 0.0, FRAC_PI_3, 20.0, 20.0);
        let (start, end) = (c.pts[1], c.pts[c.pts.len() - 2]);
        let direction = (end - start).normalize();
        for p in &c.pts[1..c.pts.len() - 1] {
            let off_line = (*p - start).perp_dot(direction).abs();
            assert!(off_line < 0.6, "strays {} px from a straight line", off_line);
        }
    }

    #[test]
    fn strand_keeps_clear_of_the_node_and_inside_the_midpoints() {
        let center = vec2(0.0, 0.0);
        for phi in [FRAC_PI_3, 2.0 * FRAC_PI_3, PI, 4.0 * FRAC_PI_3, 5.0 * FRAC_PI_3, TAU] {
            let c = corner_curve(center, 1.0, phi, 20.0, 20.0);
            for p in &c.pts {
                let r = p.length();
                assert!(r > 0.55 * 20.0 && r < 20.0 + 1.0, "phi {} radius {}", phi, r);
            }
        }
    }

    #[test]
    fn strands_pass_straight_through_a_crossing() {
        // the curve leaving an edge at one end node and the one leaving it at the other end
        // node are the two halves of one strand: their start tangents must be opposite
        let a = corner_curve(vec2(0.0, 0.0), 0.0, PI, 20.0, 20.0);
        let b = corner_curve(vec2(40.0, 0.0), PI, PI, 20.0, 20.0);
        assert!(close(a.pts[1], b.pts[1], 1e-3));
        assert!(close(a.normals[0], -b.normals[0], 1e-4));
    }

    #[test]
    fn one_curve_per_corner() {
        let mut net = Network::triangular(Rect::new(0., 0., 400., 300.), 40.);
        let center = net.node_id(4, 2);
        // a lone edge: one loop round each end
        net.active[net.incident[center][0]] = true;
        assert_eq!(corner_curves(&net).len(), 2);
        // a six-edge star: six corners at the hub and a loop at each tip
        net.clear();
        for e in net.incident[center].clone() {
            net.active[e] = true;
        }
        assert_eq!(corner_curves(&net).len(), 12);
    }

    #[test]
    fn profile_peaks_at_the_edge_midpoints() {
        let spokes = [(-2.0, 20.0), (0.5, 22.0), (1.6, 18.0)];
        let p = profile(&spokes);
        for (theta, a) in spokes {
            assert!((radius_at(&p, theta) - a).abs() < 1e-2);
            assert!((radius_at(&p, theta + TAU) - a).abs() < 1e-2);
            assert!(radius_at(&p, theta + 0.2) < a);
            assert!(radius_at(&p, theta - 0.2) < a);
        }
        assert_eq!(radius_at(&profile(&[]), 1.0), 0.0);
    }

    #[test]
    fn profile_angles_never_run_backwards() {
        for spokes in [vec![(0.3, 20.0)], vec![(-3.0, 20.0), (-1.0, 20.0), (0.0, 20.0), (3.0, 20.0)]] {
            let p = profile(&spokes);
            assert!(p.windows(2).all(|w| w[1].0 >= w[0].0 - 1e-5));
            assert!((p.last().unwrap().0 - p[0].0 - TAU).abs() < 1e-4);
        }
    }
}
