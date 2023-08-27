#![warn(clippy::pedantic)]

mod parse;
mod reduce;
mod type_check;

use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;

use crate::parse::LambdaTerm;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    file: PathBuf,
    #[arg(short, long)]
    arg: Option<PathBuf>,

    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    // Read a lambda term from the file supplied by the user.
    let lambda_term = read_to_string(&cli.file).map_or_else(
        |e| {
            eprintln!("Unable to open file {}: {}", cli.file.display(), e);
            exit(1);
        },
        |s| LambdaTerm::from_str(&s),
    );

    // If an argument was supplied, apply it to the required term.
    let lambda_term = if let Some(path) = cli.arg {
        let arg = read_to_string(&path).map_or_else(
            |e| {
                eprintln!("Unable to open file {}: {}", path.display(), e);
                exit(1);
            },
            |s| LambdaTerm::from_str(&s),
        );
        LambdaTerm::Application {
            function: Box::new(lambda_term),
            argument: Box::new(arg),
        }
    } else {
        lambda_term
    };

    let lambda_term_type = lambda_term.get_type().unwrap_or_else(|e| {
        eprintln!("Term {lambda_term} is not well-typed: {e}");
        exit(1);
    });

    // Compute the β-reduction of the lambda term.
    let lambda_term = lambda_term.beta_reduce();

    // Print the β-reduced lambda term. In debug mode, this will print the term in its derived
    // debug format to simplify debugging. When not in debug mode, variables will have their de
    // Bruijn indices replaced with human-readable names. The output format will always be parsable
    // as a valid lambda term, so computations can be chained together.
    if cli.debug {
        println!("({lambda_term:?}):{lambda_term_type:?}");
    } else {
        println!("({lambda_term}):{lambda_term_type}");
    }
}
