use anyhow::Result;

mod cli;
mod config;
mod extraction;
mod table;

fn main() -> Result<()> {
    cli::run()
}
