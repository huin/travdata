use anyhow::Result;

mod cli;
mod clock;
mod commontext;
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
mod util;
mod v8wrapper;

fn main() -> Result<()> {
    cli::run()
}
