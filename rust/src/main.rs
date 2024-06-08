use anyhow::Result;

mod cli;
mod commontext;
mod config;
mod extraction;
mod filesio;
mod fmtutil;
mod gui;
mod table;
#[cfg(test)]
mod testutil;

fn main() -> Result<()> {
    cli::run()
}
