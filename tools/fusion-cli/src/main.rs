use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod config;

use commands::{
    AiCommands, ConfigCommands, CryptoCommands, DeployCommands, NeuralSealSubcommand, ProviderSubcommand,
    PkgCommands, QuantumCommands, SourceSubcommand,
};

#[derive(Parser)]
#[command(
    name = "fusion",
    about = "Fusion v2.0 Vortex - Build, Run, and Manage Fusion Projects",
    version
)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Fusion project
    Init {
        /// Project name
        name: String,
        /// Template to use (default, gui, cli, lib)
        #[arg(short, long, default_value = "default")]
        template: String,
    },

    /// Build the project
    Build {
        /// Target: native or wasm
        #[arg(default_value = "native")]
        target: String,
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Compile and run a file
    Run {
        /// File to run
        file: Option<String>,
        /// Language runtime to use (python, javascript, java, rust, c, cpp, fusion)
        #[arg(short, long)]
        lang: Option<String>,
        /// Auto-detect language and run with appropriate runtime
        #[arg(long)]
        polyglot: bool,
    },

    /// Run tests
    Test {
        /// Specific test name filter
        #[arg(short, long)]
        filter: Option<String>,
        /// Run tests in parallel
        #[arg(short, long, default_value_t = true)]
        parallel: bool,
    },

    /// Format code
    Fmt {
        /// Check only, don't modify
        #[arg(long)]
        check: bool,
    },

    /// Lint code
    Lint {
        /// Fix auto-fixable issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Generate documentation
    Doc {
        /// Open in browser after generating
        #[arg(short, long)]
        open: bool,
    },

    /// Clean build artifacts
    Clean {
        /// Also clean cache
        #[arg(short, long)]
        cache: bool,
    },

    /// AI-powered commands
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },

    /// Package management
    Pkg {
        #[command(subcommand)]
        command: PkgCommands,
    },

    /// Quantum computing commands
    Quantum {
        #[command(subcommand)]
        command: QuantumCommands,
    },

    /// Deploy commands
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },

    /// Post-quantum cryptography commands
    Crypto {
        #[command(subcommand)]
        command: CryptoCommands,
    },

    /// Source Files management commands
    Source {
        #[command(subcommand)]
        command: SourceSubcommand,
    },

    /// NeuralSeal post-quantum cryptography commands
    Neuralseal {
        #[command(subcommand)]
        command: NeuralSealSubcommand,
    },

    /// AI model provider management
    Providers {
        #[command(subcommand)]
        command: ProviderSubcommand,
    },

    /// View and edit Fusion.toml configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Show project info
    Info,

    /// Show Fusion version and environment
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => commands::init::run(&name, &template, cli.json).await,
        Commands::Build { target, release } => commands::build::run(&target, release, cli.verbose, cli.json).await,
        Commands::Run { file, lang, polyglot } => commands::run::run(file.as_deref(), lang.as_deref(), polyglot, cli.verbose).await,
        Commands::Test { filter, parallel } => commands::test::run(filter.as_deref(), parallel, cli.json).await,
        Commands::Fmt { check } => commands::fmt::run(check, cli.json).await,
        Commands::Lint { fix } => commands::lint::run(fix, cli.json).await,
        Commands::Doc { open } => commands::doc::run(open, cli.json).await,
        Commands::Clean { cache } => commands::clean::run(cache, cli.json).await,
        Commands::Ai { command } => commands::ai::run(command, cli.json).await,
        Commands::Pkg { command } => commands::pkg::run(command, cli.json).await,
        Commands::Quantum { command } => commands::quantum::run(command, cli.json).await,
        Commands::Deploy { command } => commands::deploy::run(command, cli.json).await,
        Commands::Crypto { command } => commands::crypto::run(command, cli.json).await,
        Commands::Source { command } => commands::source::run(command, cli.json).await,
        Commands::Neuralseal { command } => commands::neuralseal::run(command, cli.json).await,
        Commands::Providers { command } => commands::providers::run(command, cli.json).await,
        Commands::Config { command } => commands::config::run(command, cli.json).await,
        Commands::Info => commands::info::run(cli.json).await,
        Commands::Version => {
            println!("{} v{}", "fusion".green().bold(), env!("CARGO_PKG_VERSION"));
            println!("  Runtime: Fusion v2.0 Vortex");
            Ok(())
        }
    };

    if let Err(e) = &result {
        eprintln!("{} {}", "error:".red().bold(), e);
    }

    result
}
