# Conway's Game of Life

Welcome to my CLI version of Conway's Game of Life! I've written an implementation of Conway's Game of Life before ([see here](https://github.com/devblocky/conway-gol)), but I decided to revisit this age-old computer science problem with new insights, new motiviations, and in a new language.

## Features

- **CLI**: Supports a number of arguments to benchmark and customize the program's starting conditions
- **State Import/Export**: Easily import initial game states or export current ones for sharing or later use
- **Optimized for Speed**: Leveraging the power of Rust for a blazing fast simulation experience

## Optimization & Speed

In traditional implementations of Conway's Game of Life, determining the next state of a cell often involves examining its neighbors and then applying the game's rules. On an infinite grid, that means binary or linear searches that are computationally expensive.

Parallel Row Cursors come in as a game-changer in this context. Although the cell states are still searched sequentially, the reliance on multiple binary or linear searches is almost completely eliminated. Instead of repeatedly searching through data structures to find the states of neighboring cells, we use a scanning 3x3 cursor.

To see the implementation of the parallel cursors, check out [`crate::engine::scan`](https://github.com/DevBlocky/cgolrs/blob/main/src/engine/scan.rs).

### Mutli-threading

With the foundation laid by the current architecture, introducing multithreading is a straightforward next step. Since this implementation is only single-threaded, partitioning the grid by rows and assigning each segment to a separate thread would drastically increase the speed of computation. Maybe I'll revisit this in the future to add this.

## Installation

1. Clone the repository:
```
git clone https://github.com/DevBlocky/cgolrs.git
```

2. Navigate to the project directory:
```
cd cgolrs
```

3. Run the project:
```
cargo run --release
```

### Using Console Mode

```
cargo run --release -- -c
```

### Importing a State

```
cargo run --release -- -c -i file.rle
```

### Exporting a State

```
cargo run --release -- -c -g1000 -o file.rle
```

For more options, use the help flag:
```
cargo run --release -- --help
```
