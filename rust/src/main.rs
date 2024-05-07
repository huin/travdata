use anyhow::Result;

mod cli;
mod config;
mod extraction;
mod filesio;
mod table;

fn main() -> Result<()> {
    cli::run()
}
