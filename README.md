# [game-solver](https://tristan-f-r.github.io/game-solver)

![Crates.io](https://img.shields.io/crates/v/game-solver)
[![Discord](https://img.shields.io/discord/1140401094338556009)](https://discord.gg/VjbCyaX29C)

Solve any sequential game. These are currently restricted to [Combinatorial Games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) - or n-player perfect-information games.

More in-depth information can be found in [the book](https://tristan-f-r.github.io/game-solver/book).

## Background

[Game Theory](https://en.wikipedia.org/wiki/Game_theory) is a general study of games. Many of these games are solved without rigirous computation (for example, where [impartial](https://en.wikipedia.org/wiki/Impartial_game) [combinatorial games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) are solved by generalizing the game to Nim).

However, computation is required to strongly solve to more complex games. This is where the `game-solver` comes in.

## Contribute

Rust nightly is required.

If you want to contribute, new game implementations would be greately appreciated!
The more examples of games that are provided, the more examples that can be used
for benchmarks, analysis, and further optimization.

Any new visual representations for games that don't exist on the [app](https://tristan-f-r.github.io/game-solver/app/) would also be great!

### Profiling

Recommended profiling tools:

#### Flamegraph

`cargo install flamegraph` (requires linux `perf` or windows `dtrace`)

### Unix

```sh
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example <example> -- <args>
```

### Windows

```ps
$env:CARGO_PROFILE_RELEASE_DEBUG='true'; cargo flamegraph --example <example> -- <args>; $env:CARGO_PROFILE_RELEASE_DEBUG=$null
```
