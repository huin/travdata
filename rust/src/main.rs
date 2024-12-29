use anyhow::Result;

mod cli;
mod commontext;
mod config;
mod distpaths;
mod extraction;
mod filesio;
mod fmtutil;
mod gui;
mod mpscutil;
mod table;
mod template;
#[cfg(test)]
mod testutil;

fn main() -> Result<()> {
    cli::run()
}
