use crate::utils::runner::{compile_cpp, run_with_input};
use crate::Problem;
use anyhow::Result;
use std::time::Duration;

pub async fn run_tests(problem: Problem, source_file: &str) -> Result<()> {
    // Try to compile first
    if !compile_cpp(source_file)? {
        eprintln!("Compilation failed");
        return Ok(());
    }

    let executable = format!("{}.out", source_file);
    let time_limit = Duration::from_secs_f64(problem.time_limit);

    for (i, test) in problem.test_cases.iter().enumerate() {
        let result = run_with_input(&executable, &test.input, time_limit)?;

        if !result.success {
            eprintln!(
                "Test case {} failed:\nTime: {:?}\nInput:\n{}\nExpected:\n{}\nGot:\n{}",
                i + 1,
                result.execution_time,
                test.input,
                test.expected_output,
                result.output
            );
            continue;
        }

        if result.output.trim() != test.expected_output.trim() {
            eprintln!(
                "Test case {} failed:\nInput:\n{}\nExpected:\n{}\nGot:\n{}",
                i + 1,
                test.input,
                test.expected_output,
                result.output
            );
        } else {
            println!("Test case {} passed in {:?}", i + 1, result.execution_time);
        }
    }

    Ok(())
}
