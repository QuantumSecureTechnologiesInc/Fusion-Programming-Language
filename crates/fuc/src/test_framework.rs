//! Fusion Test & Benchmarking Framework
//! Addresses: Limited testing infrastructure, no fuzzing, no benchmarking.
//!
//! Provides a unified test runner for the compiler to self-test AST nodes,
//! lowering phases, and standard library components.
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed(FString),
    Ignored,
}

pub struct TestCase {
    pub name: FString,
    pub func: fn() -> Result<(), FString>, // Simulated function pointer
    pub is_benchmark: FBool,
}

pub struct TestRunner {
    tests: FVec<TestCase>,
    passes: FSize,
    failures: FSize,
}


impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            passes: 0,
            failures: 0,
        }
    }

    /// Registers a new test case.
    pub fn register(&mut self, name: FString, func: fn() -> Result<(), FString>) {
        self.tests.push(TestCase {
            name,
            func,
            is_benchmark: false,
        });
    }

    /// Executes all registered tests.
    pub fn run_all(&mut self) {
        // printf("Running %d tests...\n", self.tests.len());
        
        for test in &self.tests {
            if test.is_benchmark { continue; }
            
            let result = (test.func)();
            
            match result {
                Ok(_) => {
                    self.passes += 1;
                    // printf("test %s ... ok\n", test.name);
                }
                Err(_e) => {
                    self.failures += 1;
                    // printf("test %s ... FAILED\n", test.name);
                    // printf("  Error: %s\n", e);
                }
            }
        }
        
        // printf("\nTest result: %s. %d passed; %d failed;\n", 
        //    if self.failures == 0 { "ok" } else { "FAILED" }, 
        //    self.passes, self.failures);
    }
}