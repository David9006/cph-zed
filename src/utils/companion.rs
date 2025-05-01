use anyhow::Result;
use serde::Deserialize;
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::Problem;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompanionInput {
    name: String,
    group: String,
    url: String,
    interactive: bool,
    memory_limit: i32,
    time_limit: f64,
    tests: Vec<CompanionTest>,
}

#[derive(Deserialize)]
struct CompanionTest {
    input: String,
    output: String,
}

#[derive(Clone)]
pub struct CompanionServer {
    problems: Arc<Mutex<Vec<Problem>>>,
}

impl CompanionServer {
    pub fn new() -> Self {
        Self {
            problems: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:27121").await?;
        let problems = self.problems.clone();

        tokio::spawn(async move {
            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let problems = problems.clone();

                    tokio::spawn(async move {
                        let mut buf = Vec::new();
                        if let Ok(_) = tokio::io::AsyncReadExt::read_to_end(&mut socket, &mut buf).await {
                            if let Ok(input) = serde_json::from_slice::<CompanionInput>(&buf) {
                                let problem = Problem {
                                    name: input.name,
                                    test_cases: input.tests.into_iter()
                                        .map(|t| crate::TestCase {
                                            input: t.input,
                                            expected_output: t.output,
                                        })
                                        .collect(),
                                    time_limit: input.time_limit,
                                    memory_limit: input.memory_limit,
                                };

                                let mut problems = problems.lock().await;
                                problems.push(problem);
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }

    pub async fn get_latest_problem(&self) -> Option<Problem> {
        let mut problems = self.problems.lock().await;
        problems.pop()
    }
}
