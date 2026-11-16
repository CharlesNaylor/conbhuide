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

/// The strand through the corner at `center` that starts on the edge at angle `theta` (half
/// length `a_start`) and sweeps `phi` radians to the next edge (half length `a_end`).
pub fn corner_curve(center: Vec2, theta: f32, phi: f32, a_start: f32, a_end: f32) -> Curve {
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

    let mut pts: Vec<Vec2> = polar_pts
        .iter()
        .map(|&(angle, r)| center + polar(angle).0 * r)
        .collect();
    let mut normals: Vec<Vec2> = (0..pts.len())
        .map(|k| {
            let before = pts[k.saturating_sub(1)];
            let after = pts[(k + 1).min(pts.len() - 1)];
            perp((after - before).normalize())
        })
        .collect();

    // the ends are known exactly; extend them slightly so adjoining pieces overlap
    let (radial, around) = polar(theta);
    let start_tangent = -radial * BETA.cos() + around * BETA.sin();
    let (radial, around) = polar(theta + phi);
    let end_tangent = radial * BETA.cos() + around * BETA.sin();
    let last = pts.len() - 1;
    normals[0] = perp(start_tangent);
    normals[last] = perp(end_tangent);
    pts.insert(0, pts[0] - start_tangent * OVERLAP);
    normals.insert(0, normals[0]);
    pts.push(pts[last + 1] + end_tangent * OVERLAP);
    normals.push(normals[last + 1]);
    Curve { pts, normals }
}

/// One curve per corner of the active graph.
pub fn corner_curves(net: &Network) -> Vec<Curve> {
    let mut curves = vec![];
    for (node, incident) in net.incident.iter().enumerate() {
        let center = net.nodes[node];
        // (angle, half length) of each active edge, in angular order
        let mut spokes: Vec<(f32, f32)> = incident
            .iter()
            .filter(|e| net.active[**e])
            .map(|e| {
                let d = net.nodes[net.other_end(*e, node)] - center;
                (d.y.atan2(d.x), d.length() / 2.0)
            })
            .collect();
        spokes.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
        for k in 0..spokes.len() {
            let (theta, a_start) = spokes[k];
            let (next_theta, a_end) = spokes[(k + 1) % spokes.len()];
            let mut phi = (next_theta - theta).rem_euclid(TAU);
            if spokes.len() == 1 {
                phi = TAU;
            }
            curves.push(corner_curve(center, theta, phi, a_start, a_end));
        }
    }
    curves
}

fn draw_ribbon(curve: &Curve, points: usize, width: f32, color: Color) {
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

pub fn draw_network_knot(net: &Network, style: &KnotStyle) {
    let width = net.spacing * style.strand_width;
    let gap = net.spacing * style.gap;
    let curves = corner_curves(net);
    for curve in &curves {
        draw_ribbon(curve, curve.pts.len(), width, style.strand_color);
    }
    // The start of every curve passes over the other strand at its crossing: cut that strand
    // back, then lay the start of the curve over the cut. The strands cross at 2 * BETA.
    let crossing = 2.0 * BETA;
    let cover = (width / 2.0) / crossing.sin() + (width / 2.0 + gap) / crossing.tan().abs() + 1.5;
    let covers: Vec<usize> = curves.iter().map(|c| points_within(c, cover)).collect();
    for (curve, points) in curves.iter().zip(&covers) {
        draw_ribbon(curve, *points, width + 2.0 * gap, style.background);
    }
    for (curve, points) in curves.iter().zip(&covers) {
        draw_ribbon(curve, *points, width, style.strand_color);
    }
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
}
