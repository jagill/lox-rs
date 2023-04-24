use anyhow::{bail, Result as AnyResult};
use lox::Lox;
use std::io::Write;

fn main() -> AnyResult<()> {
    let mut args = std::env::args();
    let _prog_name = args.next().expect("Expected program name in args");
    let script_path_opt = args.next();
    match args.next() {
        None => (),
        Some(_) => bail!("Usage: lox [script]"),
    }
    if let Some(script_path) = script_path_opt {
        run_file(&script_path)
    } else {
        run_prompt()
    }
}

fn run_file(script_path: &str) -> AnyResult<()> {
    let contents = std::fs::read_to_string(script_path)?;
    let mut runtime = Lox::new();
    runtime.run(&contents)?;
    Ok(())
}

fn run_prompt() -> AnyResult<()> {
    let mut runtime = Lox::new();
    do_prompt()?;
    let lines = std::io::stdin().lines();
    for line_res in lines {
        let line = line_res?;
        runtime.run(&line)?;
        do_prompt()?;
    }
    Ok(())
}

fn do_prompt() -> AnyResult<()> {
    print!("> ");
    std::io::stdout().flush()?;
    Ok(())
}
