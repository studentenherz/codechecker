use core::str;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use clap::{builder::styling::AnsiColor, builder::Styles, Args, Parser, Subcommand};
use serde::Serialize;
use tqdm::{Iter, Style};

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
    /// Listen to connections for the judgment requests
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Check a program directly
    Check(CheckArgs),
    /// Listen to the binded socket
    Listen(ListenArgs),
}

#[derive(Args, Debug)]
struct CheckArgs {
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

#[derive(Debug, Args)]
struct ListenArgs {
    /// Socket address for incomming connections
    addr: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Subcommands::Check(CheckArgs {
            exe,
            time,
            memory,
            input,
        }) => {
            let cli_input = &input;

            if let Some(input) = cli_input.input.as_ref() {
                let output = cli_input.output.as_ref().expect(
                    "This should not had happened, --input and --output args require each other",
                );

                let checker = LinesChecker::new(output);

                if let Ok(verdict) = judge(&exe, None, input, time, memory, checker) {
                    println!("{:?}", verdict);
                }
            } else {
                let directory = cli_input
                    .directory
                    .as_ref()
                    .expect("This should not had happened, it no --input was given the --directory option should");

                let numbers = sorted_list_numbers_in_folder(directory).unwrap();

                let mut max_time: u64 = 0;
                let mut max_memory: u64 = 0;
                let mut res: Option<(ProblemVerdict, u32)> = None;

                for num in numbers
                    .into_iter()
                    .tqdm()
                    .desc(Some("Testing..."))
                    .width(Some(100))
                    .style(Style::Balloon)
                {
                    let input = format!("{}/{}.in", directory, num);
                    let output = format!("{}/{}.out", directory, num);

                    let checker = LinesChecker::new(&output);

                    match judge(&exe, None, &input, time, memory, checker) {
                        Ok(ProblemVerdict::Accepted { time, memory }) => {
                            max_time = std::cmp::max(max_time, time);
                            max_memory = std::cmp::max(max_memory, memory);
                        }
                        Ok(err_verdict) => {
                            res = Some((err_verdict, num));
                            break;
                        }
                        Err(err) => panic!("{:?}", err),
                    }
                }

                if res.is_none() {
                    res = Some((
                        ProblemVerdict::Accepted {
                            time: max_time,
                            memory: max_memory,
                        },
                        0,
                    ));
                }

                match res {
                    Some((ProblemVerdict::Accepted { time, memory }, _)) => {
                        println!("Accepted time = {}, memory = {}", time, memory)
                    }
                    Some((verdict, test_case)) => {
                        println!("{:?} on test case {}", verdict, test_case)
                    }
                    None => println!("WTF? Why are we here, this shouldn't be happening"),
                }
            }
        }

        Subcommands::Listen(ListenArgs { addr }) => {
            let listener =
                TcpListener::bind(&addr).expect("Couldn't bind to the given socket address");

            println!("Listening in {addr:?}");
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    let mut buf = [0; 1024];
                    if let Ok(bytes_read) = stream.read(&mut buf) {
                        let text = str::from_utf8(&buf[0..bytes_read])
                            .expect("Error parsing string from utf8");
                        let request: JudeRequest = serde_json::from_str(text)
                            .expect("Couldn't deserialize string into a judge request");

                        let JudeRequest {
                            cmd,
                            cmd_args,
                            time,
                            memory,
                            test_dir: directory,
                        } = request;

                        match sorted_list_numbers_in_folder(&directory) {
                            Ok(numbers) => {
                                let mut max_time: u64 = 0;
                                let mut max_memory: u64 = 0;
                                let mut res: Option<(ProblemVerdict, u32)> = None;

                                for num in numbers {
                                    let input = format!("{}/{}.in", directory, num);
                                    let output = format!("{}/{}.out", directory, num);

                                    let checker = LinesChecker::new(&output);

                                    send(&mut stream, &JudgeResponse::test_case(num));
                                    match judge(
                                        &cmd,
                                        cmd_args.clone(),
                                        &input,
                                        time,
                                        memory,
                                        checker,
                                    ) {
                                        Ok(ProblemVerdict::Accepted { time, memory }) => {
                                            max_time = std::cmp::max(max_time, time);
                                            max_memory = std::cmp::max(max_memory, memory);
                                        }
                                        Ok(err_verdict) => {
                                            res = Some((err_verdict, num));
                                            break;
                                        }
                                        Err(err) => {
                                            let response = JudgeResponse::error(
                                                "Error while judging, check checker's log",
                                            );
                                            send(&mut stream, &response);
                                            panic!("{:?}", err);
                                        }
                                    }
                                }

                                if res.is_none() {
                                    res = Some((
                                        ProblemVerdict::Accepted {
                                            time: max_time,
                                            memory: max_memory,
                                        },
                                        0,
                                    ));
                                }

                                match res {
                                    Some(ver) => {
                                        send(&mut stream, &JudgeResponse::ok(ver.0));
                                    }
                                    None => {
                                        send(
                                            &mut stream,
                                            &JudgeResponse::error("Unexpected error"),
                                        );

                                        println!(
                                            "WTF? Why are we here, this shouldn't be happening"
                                        )
                                    }
                                }
                            }
                            Err(err) => {
                                send(&mut stream, &JudgeResponse::error("Can't open directory"));
                                panic!("{:?}", err);
                            }
                        }
                    } else {
                        panic!("There was an error reading from the socket");
                    }
                }
                Err(e) => println!("Couldn't get client {e}"),
            }
        }
    }
}

fn send<T: ?Sized + Serialize>(stream: &mut TcpStream, response: &T) {
    stream
        .write(serde_json::to_string(&response).unwrap().as_bytes())
        .expect("Couldn't send response to caller");
}
