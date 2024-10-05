# codechecker

This is intented to be a checker for competitive pramming code. It runs the code and checks that the program stays within time and memory limits and then checks the correctness of the output.

Part of the code is heavily inspired in https://github.com/andgein/runexe, just rewrote it in Rust.

### Building

Just run

```sh
$ cargo build --release
```

then you will have the cli tool

```sh
$ cargo run -q --release -- --help 
Run a program, check if it executes within time and memory limits, and verify if the outputs are correct.

Usage: codechecker [OPTIONS] <--input <INPUT>|--output <OUTPUT>|--directory <DIRECTORY>> <EXE>

Arguments:
  <EXE>  Path to the executable to avaluate

Options:
  -t, --time <TIME>            Time limit in milliseconds [default: 1000]
  -m, --memory <MEMORY>        Memory limit in megabytes [default: 1024]
  -i, --input <INPUT>          Test input file
  -o, --output <OUTPUT>        Test correct output file
  -d, --directory <DIRECTORY>  Directory with test cases in the format #{case}.in #{case}.out
  -h, --help                   Print help
  -V, --version                Print version
```

