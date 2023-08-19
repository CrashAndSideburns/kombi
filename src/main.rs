mod parse;
mod reduce;

use std::fs::read_to_string;
use std::path::PathBuf;

use clap::Parser;

use crate::parse::{Application, LambdaTerm};

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
    let lambda_term = read_to_string(cli.file)
        .map(|s| LambdaTerm::from_str(&s))
        .unwrap();

    // If an argument was supplied, apply it to the required term.
    let lambda_term = if let Some(path) = cli.arg {
        let arg = read_to_string(path)
            .map(|s| LambdaTerm::from_str(&s))
            .unwrap();
        LambdaTerm::Application(Application::new(lambda_term, arg))
    } else {
        lambda_term
    };

    // Compute the β-reduction of the lambda term.
    let lambda_term = lambda_term.beta_reduce();

    // Print the β-reduced lambda term. In debug mode, this will print the term in its derived
    // debug format to simplify debugging. When not in debug mode, variables will have their de
    // Bruijn indices replaced with human-readable names. The output format will always be parsable
    // as a valid lambda term, so computations can be chained together.
    if cli.debug {
        println!("{:?}", lambda_term);
    } else {
        println!("{}", lambda_term);
    }
}
