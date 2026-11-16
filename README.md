# Conbhuide

Automata whose patterns are drawn as Celtic knotwork. There are two modes.

**Network** (the default). A network automaton: the state lives on the edges of a triangular
mesh. Each edge switches on or off depending on how many of the edges that share one of its
end nodes are on. The edges that are on form a planar graph, and the knot is drawn around it:
one strand crossing on every edge, with the strands swinging round the nodes in between.

**Life**. A Life-like cellular automaton. Each connected live region is drawn as a panel of
plaitwork.

Both use Mercat's construction, so every state of either automaton is a valid knot. The
geometry is described at the top of `src/mercat.rs` (any planar graph) and `src/knot.rs`
(square grid).

## Run

```sh
cargo run --release
```

| Key | Button | Action |
|---|---|---|
| `Space` | Pause / Play | pause or resume |
| `M` | Mode | switch between Network and Life |
| `N` | Rule | next rule for the current mode |
| `R` | Random | random seed |
| `S` | Symmetric | mirror-symmetric random seed |
| `C` | Graph / Cells / Knot | switch between the knot and the raw automaton |
| `G` | | show the raw automaton faintly under the knot |
| `J` | | Network: displace the mesh nodes at random, for a hand-drawn look |
| `P` | | save `conbhuide.png` |
| click | | flip the nearest edge (Network) or cell (Life) |

When the pattern stops changing it is reseeded automatically.

Network rules: Islands, Lace, Weave, Bloom. Life rules: Life, Maze, Day & Night, Coral.
`B.../S...` lists the neighbour counts at which an edge or cell is born or survives.

## Save a picture without the GUI

```sh
cargo run -- --snapshot out.png net:weave 60       # network rule, fixed seed, 60 generations
cargo run -- --snapshot out.png net 60 --jitter    # first network rule, displaced nodes
cargo run -- --snapshot out.png maze 40            # Life-mode rule
cargo run -- --snapshot out.png netshapes          # hand-placed test graphs
cargo run -- --snapshot out.png shapes             # hand-placed Life test figures
```

## Tests

```sh
cargo test
```

## Web build

```sh
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/conbhuide.wasm js/
```

Then serve the `js` directory and open `conbhuide.html`.
