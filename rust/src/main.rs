use anyhow::Result;

mod cli;
mod config;
mod extraction;
mod filesio;
mod table;
#[cfg(test)]
mod testutil;

fn main() -> Result<()> {
    cli::run()
}
