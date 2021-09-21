# BoxBopper

A sokoban-style game in rust.

It includes:
- a web game (rust/wasm/javascript)
- a console game
- a tool that will create and solve levels (multithreaded and optimised, using an exhaustive search)

# boxbopperconsole (game)

### Building

Use `cargo build --release` in the `boxbopperconsole` directory to build the console game.  
(You may first need to run `cargo build --release` in the root directory to build the `boxbopperbase` library.)

### Usage

Type `U`, `D`, `L`, `R` (followed by Enter) to move around.  
Type `N` or `P` (followed by Enter) for next or previous level.  
Type \` (followed by Enter) to reset the level.  
Type `Q` (followed by Enter) to quit.  

# boxbopper-wasm-app

### Building
The web game requires node.js and webpack to build.

Use `npm run build` in the `www` directory to build the web game.  

Use `npm run start` to run the game locally, which can then be accessed from your web browser (typically http://localhost:8080/).

### Usage

Use arrow keys (or WASD) to move around. You can also move (in straight lines only) with the mouse.

Press `R` to reset, `N` for next level, `P` for previous level, or click the appropriate buttons.

# boxboppertool

### Building
Use `cargo build --release` in the `boxboppertool` directory to build the (console) tool.

### Usage
```
boxboppertool make [vars...]
boxboppertool solve [vars...]
boxboppertool speed_test [vars...]

vars for make:
  seed=n           rng seed (u32)
  width=n          level width 5-15                              default: 5
  height=n         level height 5-15                             default: 5
  box_density=n    box density 1-99                              default: 20
  wall_density=n   wall density 1-99                             default: 20
  max_depth=n      maximum depth to try to reach 1+              default: 100
vars for solve:
  max_moves=n      maximum number of moves to try 1+             default: 200
  builtin=n        builtin level to solve
  filename=f       custom level filename to solve
vars for speed_test:
  max_level=n      maximum level to test up to                   default: 20
  speed_test_read=f   filename to compare results with
  speed_test_write=f  filename to write results to
vars for all:
  verbosity=n      how much information to provide 0-2           default: 1
  threads=n        how many cpu threads to use 0=auto            default: 0
  max_maps=n       max maps to have in memory                    default: 4000000

lower max_maps to reduce memory usage (but it may not solve)
lower max_moves to improve performance (but it will not solve if more moves are required)
```

# License

Copyright 2020-2021 David Atkinson.
