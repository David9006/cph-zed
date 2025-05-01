use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zed_extension_api::{self, Extension};

mod utils {
    pub mod companion;
    pub mod runner;
}
mod commands;

use utils::companion::CompanionServer;

#[derive(Default)]
pub struct CphExtension {
    companion_server: Option<Arc<CompanionServer>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TestCase {
    input: String,
    expected_output: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Problem {
    name: String,
    test_cases: Vec<TestCase>,
    time_limit: f64,
    memory_limit: i32,
}

impl CphExtension {
    async fn handle_test(companion_server: Arc<CompanionServer>) -> Result<()> {
        if let Some(problem) = companion_server.get_latest_problem().await {
            commands::test::run_tests(problem, "test.cpp").await?;
        } else {
            eprintln!("No problem loaded. Use Competitive Companion to load a problem first.");
        }
        Ok(())
    }
}

impl Extension for CphExtension {
    fn new() -> Self {
        // Start the companion server in a background task
        let companion_server = Arc::new(CompanionServer::new());
        let companion_server_clone = companion_server.clone();

        tokio::spawn(async move {
            if let Err(e) = companion_server_clone.start().await {
                eprintln!("Failed to start companion server: {}", e);
            }
        });

        Self {
            companion_server: Some(companion_server),
        }
    }
}

#[no_mangle]
pub extern "C" fn cph_test(extension: &mut CphExtension) -> Result<()> {
    if let Some(companion_server) = &extension.companion_server {
        let server = companion_server.clone();
        futures::executor::block_on(CphExtension::handle_test(server))
    } else {
        eprintln!("Companion server not initialized");
        Ok(())
    }
}

zed_extension_api::register_extension!(CphExtension);
