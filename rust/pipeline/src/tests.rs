use googletest::prelude::*;

use crate::{systems, tabula_wrapper::singlethreaded, testutil::TABULA_VM};

#[gtest]
fn test_e2e_small_pipeline() -> anyhow::Result<()> {
    let tabula_extractor = {
        let jvm = TABULA_VM.as_ref().unwrap();
        let java_env = jvm.attach()?;
        singlethreaded::SingleThreadedTabulaExtractor::new(java_env)
    };
    let tabula_system = systems::TabulaPdfExtractTableSystem::new(&tabula_extractor);

    let system = Metasystem::new();

    Ok(())
}
