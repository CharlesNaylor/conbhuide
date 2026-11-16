# Conbhuide

A Life-like cellular automaton whose live regions are drawn as Celtic knotwork.

Every live cell is a vertex; live cells that share a side are joined by an edge, and each edge
carries one strand crossing. Where a live cell faces a dead one, the strand turns back around
the cell. This is Mercat's construction applied to the grid, so every state of the automaton
is a valid knot. The geometry is described at the top of `src/knot.rs`.

## Run

```sh
cargo run --release
```

| Key | Button | Action |
|---|---|---|
| `Space` | Pause / Play | pause or resume |
| `C` | Cells / Knot | switch between the knot and the raw cells |
| `G` | | show the cells faintly under the knot |
| `N` | Rule | next rule: Life, Maze, Day & Night, Coral |
| `R` | Random | random seed |
| `S` | Symmetric | four-fold symmetric random seed |
| `P` | | save `conbhuide.png` |
| click | | flip a cell |

When the pattern stops changing it is reseeded automatically.

## Save a picture without the GUI

```sh
cargo run -- --snapshot out.png            # hand-placed test figures
cargo run -- --snapshot out.png maze 40    # Maze rule, fixed seed, 40 generations
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
