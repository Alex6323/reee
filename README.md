# reee
Implementation of the EEE model in Rust. If you don't know what this is, then read this [ELI9 blog post](https://blog.iota.org/eli9-the-qubic-computation-model-623417f94777).

## Examples

To get and idea about the performance of EEE we use Convey's Game of Life. 

Currently you can only run the Game-of-Life reference implementation:
```bash
    $ cargo r --release --example gol_ref random
```

There are also two universes with a fixed initial state, `glider_gun`, and the other one I forgot unfortunately. Maybe you'll find out somehow. You might adjust your terminal size so that the universes can be fully displayed.
