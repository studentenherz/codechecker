use clap::{builder::styling::AnsiColor, builder::Styles, Args, Parser};

use codechecker::*;

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Green.on_default().bold())
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug)]
#[command(
    author = "Michel Romero",
    version,
    about = "Run a program, check if it executes within time and memory limits, and verify if the outputs are correct.",
    long_about = None
)]
#[command(styles=cli_styles())]
struct Cli {
    /// Path to the executable to avaluate
    exe: String,

    /// Time limit in milliseconds
    #[arg(short, long, default_value = "1000")]
    time: u64,

    /// Memory limit in megabytes
    #[arg(short, long, default_value = "1024")]
    memory: u64,

    #[command(flatten)]
    input: InputArgs,
}

#[derive(Args, Debug)]
#[group(required = true)]
struct InputArgs {
    /// Test input file
    #[arg(short, long, conflicts_with = "directory", requires = "output")]
    input: Option<String>,

    /// Test correct output file
    #[arg(short, long, conflicts_with = "directory", requires = "input")]
    output: Option<String>,

    /// Directory with test cases in the format #{case}.in #{case}.out
    #[arg(short, long, conflicts_with_all = ["input", "output"])]
    directory: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let cli_input = &cli.input;

    if let Some(input) = cli_input.input.as_ref() {
        let output = cli_input
            .output
            .as_ref()
            .expect("This should not had happened, --input and --output args require each other");

        let checker = LinesChecker::new(output);

        if let Ok(verdict) = judge(&cli.exe, input, cli.time, cli.memory, checker) {
            println!("{:?}", verdict);
        }
    } else {
        let dir = cli_input.directory.as_ref().expect(
            "This should not had happened, it no --input was given the --directory option should",
        );

        match judge_all(&cli.exe, dir, cli.time, cli.memory) {
            Ok((ProblemVerdict::Accepted { time, memory }, _)) => {
                println!("Accepted time = {}, memory = {}", time, memory)
            }
            Ok((verdict, test_case)) => println!("{:?} on test case {}", verdict, test_case),
            Err(err) => println!("{:?}", err),
        }
    }
}
