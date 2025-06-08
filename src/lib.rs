use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zed_extension_api::{self, Extension, SlashCommand, SlashCommandOutput, SlashCommandOutputSection};

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

    pub fn register_commands(&self) {
        SlashCommand::new("cph.test")
            .description("Run test cases for the current file")
            .handler(|_args| {
                let output = SlashCommandOutput::new()
                    .title("Test Results")
                    .section(SlashCommandOutputSection::new("Running tests...").info());

                // Replace simulated logic with actual test execution
                if let Err(e) = futures::executor::block_on(commands::test::run_tests(
                    Problem {
                        name: "Example Problem".to_string(),
                        test_cases: vec![
                            TestCase {
                                input: "1\n2\n".to_string(),
                                expected_output: "3\n".to_string(),
                            },
                        ],
                        time_limit: 2.0,
                        memory_limit: 256,
                    },
                    "test.cpp",
                )) {
                    output.section(SlashCommandOutputSection::new(&format!("Error: {}", e)).error());
                } else {
                    output.section(SlashCommandOutputSection::new("All tests passed").success());
                }

                Ok(output)
            })
            .register();

        SlashCommand::new("cph.compile")
            .description("Compile the current solution")
            .handler(|_args| {
                let output = SlashCommandOutput::new()
                    .title("Compilation Results")
                    .section(SlashCommandOutputSection::new("Compiling...").info());

                // Replace simulated logic with actual compilation
                if let Err(e) = crate::utils::runner::compile_cpp("test.cpp") {
                    output.section(SlashCommandOutputSection::new(&format!("Compilation failed: {}", e)).error());
                } else {
                    output.section(SlashCommandOutputSection::new("Compilation successful").success());
                }

                Ok(output)
            })
            .register();
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

pub struct SlashCommandWrapper;

impl SlashCommandWrapper {
    pub fn new(name: &str) -> SlashCommand {
        SlashCommand {
            name: name.to_string(),
            tooltip_text: "".to_string(),
            requires_argument: false,
        }
    }
}

pub struct SlashCommandOutputWrapper;

impl SlashCommandOutputWrapper {
    pub fn new(text: &str) -> SlashCommandOutput {
        SlashCommandOutput {
            text: text.to_string(),
        }
    }
}

pub struct SlashCommandOutputSectionWrapper;

impl SlashCommandOutputSectionWrapper {
    pub fn new(label: &str) -> SlashCommandOutputSection {
        SlashCommandOutputSection {
            range: None,
            label: label.to_string(),
        }
    }
}

impl SlashCommand {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            handler: None,
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&[String]) -> Result<SlashCommandOutput> + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    pub fn register(self) {
        // Register the command (implementation depends on the framework being used)
    }
}

zed_extension_api::register_extension!(CphExtension);
