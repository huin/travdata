use anyhow::Result;

mod cli;
mod commontext;
mod distpaths;
mod extraction;
mod filesio;
mod fmtutil;
mod gui;
mod table;
mod template;
mod v8wrapper;

fn main() -> Result<()> {
    cli::run()
}
