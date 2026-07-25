//! # Fusion Visual Compiler
//!
//! Natural language intent parser, architecture suggestion engine,
//! code template generation, and project scaffolding for the Fusion
//! Programming Language.
//!
//! This crate converts human-readable descriptions into structured
//! Fusion project plans and generated code scaffolding.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ──────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum VisualCompilerError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("unknown intent: {0}")]
    UnknownIntent(String),
    #[error("scaffold error: {0}")]
    ScaffoldError(String),
    #[error("template error: {0}")]
    TemplateError(String),
}

pub type Result<T> = std::result::Result<T, VisualCompilerError>;

// ──────────────────────────────────────────────
// Intent model
// ──────────────────────────────────────────────

/// The parsed intent from a natural-language description.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectIntent {
    /// Build a REST API server.
    RestApi {
        name: String,
        endpoints: Vec<EndpointSpec>,
    },
    /// Build a CLI tool.
    Cli {
        name: String,
        subcommands: Vec<String>,
    },
    /// Build a library / shared module.
    Library {
        name: String,
        exports: Vec<String>,
    },
    /// Build a data-processing pipeline.
    DataPipeline {
        name: String,
        stages: Vec<String>,
    },
    /// Generic / unrecognized intent – we still store the raw tokens.
    Generic {
        name: String,
        tokens: Vec<String>,
    },
}

/// A single REST endpoint specification derived from NL keywords.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointSpec {
    pub method: HttpMethod,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

// ──────────────────────────────────────────────
// Intent parser (keyword-matching + rule-based)
// ──────────────────────────────────────────────

/// Keyword tables used by the intent parser.
struct KeywordRules {
    intent_keywords: Vec<(&'static str, &'static str)>,
    method_keywords: Vec<(&'static str, HttpMethod)>,
}

impl Default for KeywordRules {
    fn default() -> Self {
        Self {
            intent_keywords: vec![
                ("api", "rest_api"),
                ("server", "rest_api"),
                ("endpoint", "rest_api"),
                ("http", "rest_api"),
                ("cli", "cli"),
                ("command line", "cli"),
                ("terminal", "cli"),
                ("library", "library"),
                ("lib", "library"),
                ("module", "library"),
                ("pipeline", "data_pipeline"),
                ("etl", "data_pipeline"),
                ("data flow", "data_pipeline"),
                ("stream", "data_pipeline"),
            ],
            method_keywords: vec![
                ("get", HttpMethod::Get),
                ("fetch", HttpMethod::Get),
                ("read", HttpMethod::Get),
                ("create", HttpMethod::Post),
                ("add", HttpMethod::Post),
                ("post", HttpMethod::Post),
                ("update", HttpMethod::Put),
                ("modify", HttpMethod::Put),
                ("put", HttpMethod::Put),
                ("delete", HttpMethod::Delete),
                ("remove", HttpMethod::Delete),
            ],
        }
    }
}

/// The natural-language intent parser.
pub struct IntentParser {
    rules: KeywordRules,
}

impl Default for IntentParser {
    fn default() -> Self {
        Self {
            rules: KeywordRules::default(),
        }
    }
}

impl IntentParser {
    /// Create a new parser with default keyword rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a natural-language description into a `ProjectIntent`.
    pub fn parse(&self, description: &str) -> Result<ProjectIntent> {
        let lower = description.to_lowercase();
        let tokens: Vec<String> = lower
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            return Err(VisualCompilerError::ParseError(
                "empty description".into(),
            ));
        }

        // Extract project name from first meaningful token(s).
        let name = self.extract_name(&tokens);

        // Detect intent category.
        let intent_type = self.detect_intent_type(&lower);

        match intent_type {
            "rest_api" => self.parse_rest_api(&name, &lower, &tokens),
            "cli" => self.parse_cli(&name, &tokens),
            "library" => self.parse_library(&name, &tokens),
            "data_pipeline" => self.parse_data_pipeline(&name, &tokens),
            _ => Ok(ProjectIntent::Generic {
                name,
                tokens: tokens.clone(),
            }),
        }
    }

    fn extract_name(&self, tokens: &[String]) -> String {
        // Skip filler words, take first real token(s).
        let skip = ["build", "create", "make", "a", "an", "the", "for", "with", "to", "that"];
        let name_tokens: Vec<&str> = tokens
            .iter()
            .filter(|t| !skip.contains(&t.as_str()))
            .take(2)
            .map(|s| s.as_str())
            .collect();

        if name_tokens.is_empty() {
            "fusion_project".into()
        } else {
            name_tokens.join("_")
        }
    }

    fn detect_intent_type(&self, lower: &str) -> &str {
        let mut best_intent = "generic";
        let mut best_score = 0u32;

        for (keyword, intent) in &self.rules.intent_keywords {
            if lower.contains(keyword) {
                let score = keyword.len() as u32;
                if score > best_score {
                    best_score = score;
                    best_intent = intent;
                }
            }
        }

        best_intent
    }

    fn parse_rest_api(
        &self,
        name: &str,
        _lower: &str,
        tokens: &[String],
    ) -> Result<ProjectIntent> {
        let endpoints = self.extract_endpoints(tokens);
        Ok(ProjectIntent::RestApi {
            name: name.to_string(),
            endpoints,
        })
    }

    fn extract_endpoints(&self, tokens: &[String]) -> Vec<EndpointSpec> {
        let mut endpoints = Vec::new();
        let mut current_method = HttpMethod::Get;

        for token in tokens {
            // Check for HTTP method keywords.
            for (kw, method) in &self.rules.method_keywords {
                if token == *kw {
                    current_method = method.clone();
                    break;
                }
            }

            // Check for resource-like tokens (noun-ish).
            if token.len() > 2
                && !self.rules.method_keywords.iter().any(|(kw, _)| token == *kw)
            {
                endpoints.push(EndpointSpec {
                    method: current_method.clone(),
                    path: format!("/{}", token),
                    description: format!("{} operation on {}", format!("{:?}", current_method), token),
                });
                // Reset to GET for next resource.
                current_method = HttpMethod::Get;
            }
        }

        if endpoints.is_empty() {
            endpoints.push(EndpointSpec {
                method: HttpMethod::Get,
                path: "/".into(),
                description: "Root endpoint".into(),
            });
        }

        endpoints
    }

    fn parse_cli(&self, name: &str, tokens: &[String]) -> Result<ProjectIntent> {
        // Look for verb tokens that become subcommands.
        let verbs = ["run", "build", "test", "check", "lint", "fmt", "deploy", "init", "new"];
        let subcommands: Vec<String> = tokens
            .iter()
            .filter(|t| verbs.contains(&t.as_str()))
            .cloned()
            .collect();

        let subcommands = if subcommands.is_empty() {
            vec!["run".into(), "build".into()]
        } else {
            subcommands
        };

        Ok(ProjectIntent::Cli {
            name: name.to_string(),
            subcommands,
        })
    }

    fn parse_library(&self, name: &str, tokens: &[String]) -> Result<ProjectIntent> {
        let exports: Vec<String> = tokens
            .iter()
            .filter(|t| t.len() > 2)
            .take(5)
            .cloned()
            .collect();

        Ok(ProjectIntent::Library {
            name: name.to_string(),
            exports,
        })
    }

    fn parse_data_pipeline(&self, name: &str, tokens: &[String]) -> Result<ProjectIntent> {
        // Treat non-filler tokens as pipeline stage names.
        let skip = ["build", "create", "make", "a", "an", "the", "for", "with", "to", "that", "data", "pipeline"];
        let stages: Vec<String> = tokens
            .iter()
            .filter(|t| !skip.contains(&t.as_str()) && t.len() > 2)
            .cloned()
            .collect();

        let stages = if stages.is_empty() {
            vec!["ingest".into(), "transform".into(), "load".into()]
        } else {
            stages
        };

        Ok(ProjectIntent::DataPipeline {
            name: name.to_string(),
            stages,
        })
    }
}

// ──────────────────────────────────────────────
// Architecture suggestion engine
// ──────────────────────────────────────────────

/// A suggested architecture component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchComponent {
    pub name: String,
    pub kind: ComponentKind,
    pub dependencies: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentKind {
    Module,
    Service,
    Handler,
    Database,
    Cache,
    Queue,
    Scheduler,
}

/// Suggests architecture components based on a parsed intent.
pub struct ArchitectureEngine;

impl ArchitectureEngine {
    pub fn new() -> Self {
        Self
    }

    /// Given a `ProjectIntent`, return a list of suggested components.
    pub fn suggest(&self, intent: &ProjectIntent) -> Vec<ArchComponent> {
        match intent {
            ProjectIntent::RestApi { name, endpoints } => {
                self.suggest_rest_api_arch(name, endpoints)
            }
            ProjectIntent::Cli { name, subcommands } => {
                self.suggest_cli_arch(name, subcommands)
            }
            ProjectIntent::Library { name, .. } => self.suggest_library_arch(name),
            ProjectIntent::DataPipeline { name, stages } => {
                self.suggest_pipeline_arch(name, stages)
            }
            ProjectIntent::Generic { name, .. } => vec![ArchComponent {
                name: name.clone(),
                kind: ComponentKind::Module,
                dependencies: vec![],
                description: "Root module".into(),
            }],
        }
    }

    fn suggest_rest_api_arch(&self, name: &str, endpoints: &[EndpointSpec]) -> Vec<ArchComponent> {
        let mut components = vec![
            ArchComponent {
                name: format!("{}_server", name),
                kind: ComponentKind::Service,
                dependencies: vec![format!("{}_router", name)],
                description: "HTTP server entry point".into(),
            },
            ArchComponent {
                name: format!("{}_router", name),
                kind: ComponentKind::Handler,
                dependencies: vec![format!("{}_db", name)],
                description: "Route dispatcher".into(),
            },
            ArchComponent {
                name: format!("{}_db", name),
                kind: ComponentKind::Database,
                dependencies: vec![],
                description: "Data access layer".into(),
            },
        ];

        // Add a handler per endpoint.
        for ep in endpoints {
            let handler_name = ep
                .path
                .trim_start_matches('/')
                .replace('/', "_");
            components.push(ArchComponent {
                name: format!("handler_{}", handler_name),
                kind: ComponentKind::Handler,
                dependencies: vec![format!("{}_router", name)],
                description: ep.description.clone(),
            });
        }

        components
    }

    fn suggest_cli_arch(&self, name: &str, subcommands: &[String]) -> Vec<ArchComponent> {
        let mut components = vec![ArchComponent {
            name: format!("{}_cli", name),
            kind: ComponentKind::Service,
            dependencies: subcommands.iter().map(|s| format!("cmd_{}", s)).collect(),
            description: "CLI entry point with subcommand dispatch".into(),
        }];

        for sub in subcommands {
            components.push(ArchComponent {
                name: format!("cmd_{}", sub),
                kind: ComponentKind::Handler,
                dependencies: vec![],
                description: format!("Handler for '{}' subcommand", sub),
            });
        }

        components
    }

    fn suggest_library_arch(&self, name: &str) -> Vec<ArchComponent> {
        vec![
            ArchComponent {
                name: format!("{}_core", name),
                kind: ComponentKind::Module,
                dependencies: vec![],
                description: "Core library logic".into(),
            },
            ArchComponent {
                name: format!("{}_types", name),
                kind: ComponentKind::Module,
                dependencies: vec![],
                description: "Type definitions".into(),
            },
        ]
    }

    fn suggest_pipeline_arch(&self, name: &str, stages: &[String]) -> Vec<ArchComponent> {
        let mut components = vec![ArchComponent {
            name: format!("{}_orchestrator", name),
            kind: ComponentKind::Scheduler,
            dependencies: stages.iter().map(|s| format!("stage_{}", s)).collect(),
            description: "Pipeline orchestrator".into(),
        }];

        for (i, stage) in stages.iter().enumerate() {
            let mut deps = vec![format!("{}_orchestrator", name)];
            if i > 0 {
                deps.push(format!("stage_{}", stages[i - 1]));
            }
            components.push(ArchComponent {
                name: format!("stage_{}", stage),
                kind: ComponentKind::Module,
                dependencies: deps,
                description: format!("Pipeline stage: {}", stage),
            });
        }

        // Add queue between pipeline stages.
        components.push(ArchComponent {
            name: format!("{}_queue", name),
            kind: ComponentKind::Queue,
            dependencies: vec![],
            description: "Inter-stage message queue".into(),
        });

        components
    }
}

// ──────────────────────────────────────────────
// Code template generation
// ──────────────────────────────────────────────

/// Generates Fusion source code from architecture components.
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a complete source file from a project intent.
    pub fn generate_main(&self, intent: &ProjectIntent) -> String {
        match intent {
            ProjectIntent::RestApi { name, endpoints } => {
                self.generate_rest_api_main(name, endpoints)
            }
            ProjectIntent::Cli { name, subcommands } => {
                self.generate_cli_main(name, subcommands)
            }
            ProjectIntent::Library { name, exports } => {
                self.generate_library_main(name, exports)
            }
            ProjectIntent::DataPipeline { name, stages } => {
                self.generate_pipeline_main(name, stages)
            }
            ProjectIntent::Generic { name, tokens } => {
                format!("// Generated by Fusion Visual Compiler\n// {}\n\nfn main() {{\n    print(\"Hello from {}!\")\n}}\n",
                    tokens.join(" "), name)
            }
        }
    }

    fn generate_rest_api_main(&self, name: &str, endpoints: &[EndpointSpec]) -> String {
        let mut code = format!(
            "// Generated by Fusion Visual Compiler\n// REST API: {}\n\n",
            name
        );

        for ep in endpoints {
            let func_name = ep
                .path
                .trim_start_matches('/')
                .replace('/', "_");
            code.push_str(&format!(
                "fn {}_handler(req: Request) -> Response {{\n    // {}\n    Response::ok(\"{}\")\n}}\n\n",
                func_name, ep.description, ep.path
            ));
        }

        code.push_str(&format!(
            "fn main() {{\n    let server = Server::new(\"{}\")\n",
            name
        ));

        for ep in endpoints {
            let func_name = ep
                .path
                .trim_start_matches('/')
                .replace('/', "_");
            code.push_str(&format!(
                "        .route(\"{}\", {:?}, {}_handler)\n",
                ep.path, ep.method, func_name
            ));
        }

        code.push_str("        .start()\n}\n");
        code
    }

    fn generate_cli_main(&self, name: &str, subcommands: &[String]) -> String {
        let mut code = format!(
            "// Generated by Fusion Visual Compiler\n// CLI: {}\n\n",
            name
        );

        for sub in subcommands {
            code.push_str(&format!(
                "fn cmd_{}() {{\n    print(\"Executing {} command\")\n}}\n\n",
                sub, sub
            ));
        }

        code.push_str("fn main() {\n    let args = env::args()\n");
        code.push_str("    match args.get(1) {\n");

        for sub in subcommands {
            code.push_str(&format!(
                "        Some(\"{}\") => cmd_{}(),\n",
                sub, sub
            ));
        }

        code.push_str("        _ => print(\"Usage: {} <command>\")\n    }\n}\n");
        code
    }

    fn generate_library_main(&self, name: &str, exports: &[String]) -> String {
        let mut code = format!(
            "// Generated by Fusion Visual Compiler\n// Library: {}\n\n",
            name
        );

        for export in exports {
            code.push_str(&format!(
                "pub fn {}() {{\n    // TODO: implement\n}}\n\n",
                export
            ));
        }

        if exports.is_empty() {
            code.push_str("pub fn init() {\n    // Library initialization\n}\n\n");
        }

        code
    }

    fn generate_pipeline_main(&self, name: &str, stages: &[String]) -> String {
        let mut code = format!(
            "// Generated by Fusion Visual Compiler\n// Data Pipeline: {}\n\n",
            name
        );

        for stage in stages {
            code.push_str(&format!(
                "fn stage_{}(data: Data) -> Data {{\n    // TODO: implement {} stage\n    data\n}}\n\n",
                stage, stage
            ));
        }

        code.push_str("fn main() {\n    let data = Data::load()\n");
        for stage in stages {
            code.push_str(&format!("        .pipe(stage_{})\n", stage));
        }
        code.push_str("        .save()\n}\n");
        code
    }
}

// ──────────────────────────────────────────────
// Project scaffolding
// ──────────────────────────────────────────────

/// Represents a file in a scaffolded project.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaffoldFile {
    pub path: String,
    pub content: String,
}

/// Scaffolds a full project directory structure from a `ProjectIntent`.
pub struct ProjectScaffolder;

impl ProjectScaffolder {
    pub fn new() -> Self {
        Self
    }

    /// Generate the full set of files for the project.
    pub fn scaffold(&self, intent: &ProjectIntent) -> Vec<ScaffoldFile> {
        let name = match intent {
            ProjectIntent::RestApi { name, .. }
            | ProjectIntent::Cli { name, .. }
            | ProjectIntent::Library { name, .. }
            | ProjectIntent::DataPipeline { name, .. }
            | ProjectIntent::Generic { name, .. } => name.clone(),
        };

        let generator = CodeGenerator::new();
        let mut files = vec![
            ScaffoldFile {
                path: "Fusion.toml".into(),
                content: self.generate_fusion_toml(&name),
            },
            ScaffoldFile {
                path: format!("src/main.fu"),
                content: generator.generate_main(intent),
            },
            ScaffoldFile {
                path: "README.md".into(),
                content: self.generate_readme(intent),
            },
        ];

        match intent {
            ProjectIntent::RestApi { endpoints, .. } => {
                for ep in endpoints {
                    let mod_name = ep
                        .path
                        .trim_start_matches('/')
                        .replace('/', "_");
                    files.push(ScaffoldFile {
                        path: format!("src/handlers/{}.fu", mod_name),
                        content: format!(
                            "// Handler for {} {}\n\npub fn handle(req: Request) -> Response {{\n    Response::ok(\"TODO\")\n}}\n",
                            format!("{:?}", ep.method),
                            ep.path
                        ),
                    });
                }
            }
            ProjectIntent::Cli { subcommands, .. } => {
                for sub in subcommands {
                    files.push(ScaffoldFile {
                        path: format!("src/commands/{}.fu", sub),
                        content: format!(
                            "// Command: {}\n\npub fn execute(args: Args) {{\n    // TODO: implement\n}}\n",
                            sub
                        ),
                    });
                }
            }
            ProjectIntent::Library { exports, .. } => {
                if !exports.is_empty() {
                    let mod_content: Vec<String> = exports
                        .iter()
                        .map(|e| format!("mod {};", e))
                        .collect();
                    files.push(ScaffoldFile {
                        path: "src/lib.fu".into(),
                        content: mod_content.join("\n") + "\n",
                    });
                }
            }
            ProjectIntent::DataPipeline { stages, .. } => {
                for stage in stages {
                    files.push(ScaffoldFile {
                        path: format!("src/stages/{}.fu", stage),
                        content: format!(
                            "// Pipeline stage: {}\n\npub fn process(data: Data) -> Data {{\n    // TODO: implement\n    data\n}}\n",
                            stage
                        ),
                    });
                }
            }
            _ => {}
        }

        files
    }

    fn generate_fusion_toml(&self, name: &str) -> String {
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
            name
        )
    }

    fn generate_readme(&self, intent: &ProjectIntent) -> String {
        let (title, desc) = match intent {
            ProjectIntent::RestApi { name, endpoints } => (
                name.clone(),
                format!("REST API project with {} endpoint(s)", endpoints.len()),
            ),
            ProjectIntent::Cli { name, subcommands } => (
                name.clone(),
                format!("CLI tool with {} subcommand(s)", subcommands.len()),
            ),
            ProjectIntent::Library { name, exports } => (
                name.clone(),
                format!("Library crate with {} export(s)", exports.len()),
            ),
            ProjectIntent::DataPipeline { name, stages } => (
                name.clone(),
                format!("Data pipeline with {} stage(s)", stages.len()),
            ),
            ProjectIntent::Generic { name, tokens } => (
                name.clone(),
                format!("Project: {}", tokens.join(" ")),
            ),
        };

        format!(
            "# {}\n\nGenerated by Fusion Visual Compiler.\n\n## Description\n\n{}\n\n## Build\n\n```bash\nfusion build\n```\n\n## Run\n\n```bash\nfusion run\n```\n",
            title, desc
        )
    }
}

// ──────────────────────────────────────────────
// High-level API
// ──────────────────────────────────────────────

/// Compile a natural-language description into a full project scaffold.
///
/// Returns the generated files along with the parsed intent for inspection.
pub fn compile_from_description(description: &str) -> Result<(ProjectIntent, Vec<ScaffoldFile>)> {
    let parser = IntentParser::new();
    let intent = parser.parse(description)?;

    let scaffolder = ProjectScaffolder::new();
    let files = scaffolder.scaffold(&intent);

    Ok((intent, files))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Intent Parser tests ──

    #[test]
    fn test_parse_rest_api() {
        let parser = IntentParser::new();
        let intent = parser
            .parse("Build a REST API server for users with create and delete operations")
            .unwrap();

        match &intent {
            ProjectIntent::RestApi { name, endpoints } => {
                assert!(!name.is_empty());
                assert!(!endpoints.is_empty());
            }
            _ => panic!("expected RestApi intent"),
        }
    }

    #[test]
    fn test_parse_cli() {
        let parser = IntentParser::new();
        let intent = parser
            .parse("Create a CLI tool with build and test commands")
            .unwrap();

        match &intent {
            ProjectIntent::Cli {
                name,
                subcommands,
            } => {
                assert!(!name.is_empty());
                assert!(subcommands.contains(&"build".to_string()));
                assert!(subcommands.contains(&"test".to_string()));
            }
            _ => panic!("expected Cli intent"),
        }
    }

    #[test]
    fn test_parse_library() {
        let parser = IntentParser::new();
        let intent = parser
            .parse("Make a library for math utils and parsing")
            .unwrap();

        match &intent {
            ProjectIntent::Library { name, exports } => {
                assert!(!name.is_empty());
                assert!(!exports.is_empty());
            }
            _ => panic!("expected Library intent"),
        }
    }

    #[test]
    fn test_parse_data_pipeline() {
        let parser = IntentParser::new();
        let intent = parser
            .parse("Build a data pipeline for ingestion cleaning and analysis")
            .unwrap();

        match &intent {
            ProjectIntent::DataPipeline { name, stages } => {
                assert!(!name.is_empty());
                assert!(stages.len() >= 2);
            }
            _ => panic!("expected DataPipeline intent"),
        }
    }

    #[test]
    fn test_parse_empty_description() {
        let parser = IntentParser::new();
        assert!(parser.parse("").is_err());
    }

    // ── Architecture Engine tests ──

    #[test]
    fn test_rest_api_architecture() {
        let engine = ArchitectureEngine::new();
        let intent = ProjectIntent::RestApi {
            name: "blog".into(),
            endpoints: vec![
                EndpointSpec {
                    method: HttpMethod::Get,
                    path: "/posts".into(),
                    description: "List posts".into(),
                },
                EndpointSpec {
                    method: HttpMethod::Post,
                    path: "/posts".into(),
                    description: "Create post".into(),
                },
            ],
        };

        let components = engine.suggest(&intent);
        assert!(components.len() >= 3); // server, router, db + handlers
        assert!(components.iter().any(|c| c.kind == ComponentKind::Service));
        assert!(components.iter().any(|c| c.kind == ComponentKind::Database));
    }

    #[test]
    fn test_cli_architecture() {
        let engine = ArchitectureEngine::new();
        let intent = ProjectIntent::Cli {
            name: "mytool".into(),
            subcommands: vec!["run".into(), "build".into()],
        };

        let components = engine.suggest(&intent);
        assert_eq!(components.len(), 3); // cli + 2 commands
    }

    #[test]
    fn test_pipeline_architecture() {
        let engine = ArchitectureEngine::new();
        let intent = ProjectIntent::DataPipeline {
            name: "etl".into(),
            stages: vec!["extract".into(), "transform".into(), "load".into()],
        };

        let components = engine.suggest(&intent);
        // orchestrator + 3 stages + queue = 5
        assert_eq!(components.len(), 5);
        assert!(components
            .iter()
            .any(|c| c.kind == ComponentKind::Scheduler));
        assert!(components.iter().any(|c| c.kind == ComponentKind::Queue));
    }

    // ── Code Generator tests ──

    #[test]
    fn test_generate_rest_api_code() {
        let gen = CodeGenerator::new();
        let intent = ProjectIntent::RestApi {
            name: "users".into(),
            endpoints: vec![EndpointSpec {
                method: HttpMethod::Get,
                path: "/users".into(),
                description: "List users".into(),
            }],
        };

        let code = gen.generate_main(&intent);
        assert!(code.contains("users_handler"));
        assert!(code.contains("Generated by Fusion Visual Compiler"));
    }

    #[test]
    fn test_generate_cli_code() {
        let gen = CodeGenerator::new();
        let intent = ProjectIntent::Cli {
            name: "mycli".into(),
            subcommands: vec!["run".into(), "test".into()],
        };

        let code = gen.generate_main(&intent);
        assert!(code.contains("cmd_run"));
        assert!(code.contains("cmd_test"));
    }

    // ── Project Scaffolder tests ──

    #[test]
    fn test_scaffold_rest_api() {
        let scaffolder = ProjectScaffolder::new();
        let intent = ProjectIntent::RestApi {
            name: "blog".into(),
            endpoints: vec![EndpointSpec {
                method: HttpMethod::Get,
                path: "/posts".into(),
                description: "List posts".into(),
            }],
        };

        let files = scaffolder.scaffold(&intent);
        assert!(files.iter().any(|f| f.path == "Fusion.toml"));
        assert!(files.iter().any(|f| f.path == "src/main.fu"));
        assert!(files.iter().any(|f| f.path == "README.md"));
        assert!(files.iter().any(|f| f.path.contains("handlers")));
    }

    #[test]
    fn test_scaffold_produces_valid_toml() {
        let scaffolder = ProjectScaffolder::new();
        let intent = ProjectIntent::Cli {
            name: "tool".into(),
            subcommands: vec!["run".into()],
        };

        let files = scaffolder.scaffold(&intent);
        let toml_file = files.iter().find(|f| f.path == "Fusion.toml").unwrap();
        assert!(toml_file.content.contains("name = \"tool\""));
    }

    // ── End-to-end test ──

    #[test]
    fn test_compile_from_description() {
        let (intent, files) =
            compile_from_description("Build a REST API server for products").unwrap();

        assert!(!files.is_empty());
        match &intent {
            ProjectIntent::RestApi { name, .. } => assert!(!name.is_empty()),
            _ => panic!("expected RestApi"),
        }
    }
}
