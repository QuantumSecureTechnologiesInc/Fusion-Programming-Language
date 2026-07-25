use clap::Subcommand;

pub mod ai;
pub mod build;
pub mod clean;
pub mod config;
pub mod crypto;
pub mod deploy;
pub mod doc;
pub mod fmt;
pub mod init;
pub mod info;
pub mod lint;
pub mod neuralseal;
pub mod pkg;
pub mod providers;
pub mod quantum;
pub mod run;
pub mod source;
pub mod test;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Interactive chat with an AI model
    Chat {
        /// Model name (e.g. llama3, mistral)
        model: String,
    },
    /// Code completion
    Complete {
        /// Model name
        model: String,
        /// Prompt for completion
        prompt: String,
    },
    /// Explain code in a file
    Explain {
        /// File path
        file: String,
    },
    /// Review code
    Review {
        /// File path
        file: String,
    },
    /// Generate code from a description
    Generate {
        /// Description of code to generate
        #[arg(trailing_var_arg = true)]
        description: Vec<String>,
    },
    /// List available AI models
    Models,
    /// Pull a model via Ollama
    Pull {
        /// Model name
        model: String,
    },
}

#[derive(Subcommand)]
pub enum PkgCommands {
    /// Add a dependency
    Add {
        /// Package name (name@version)
        package: String,
    },
    /// Remove a dependency
    Remove {
        /// Package name
        package: String,
    },
    /// Search packages
    Search {
        /// Search query
        query: String,
    },
    /// Publish package to registry
    Publish {
        /// Dry run without publishing
        #[arg(long)]
        dry_run: bool,
    },
    /// Update dependencies
    Update {
        /// Specific package to update
        package: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum QuantumCommands {
    /// Simulate a quantum circuit
    Simulate {
        /// Circuit file path (.qasm or .qcl)
        circuit: String,
        /// Number of shots
        #[arg(short, long, default_value_t = 1024)]
        shots: u32,
    },
    /// Run on a real quantum backend
    Run {
        /// Circuit file path
        circuit: String,
        /// Backend provider
        #[arg(short, long, value_parser = ["ibm", "aws", "azure", "google", "rigetti"])]
        backend: String,
    },
    /// List available quantum devices
    Devices {
        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DeployCommands {
    /// Deploy to a target
    Target {
        /// Deployment target: k8s, faas, cloud
        target: String,
        /// Environment (dev, staging, prod)
        #[arg(short, long, default_value = "dev")]
        env: String,
    },
    /// Check deployment status
    Status {
        /// Deployment ID
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Rollback deployment
    Rollback {
        /// Deployment ID to rollback to
        #[arg(short, long)]
        to: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CryptoCommands {
    /// Generate a hybrid PQC keypair
    Keygen {
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,
        /// Algorithm: mlkem-mldsa (default), mlkem-falcon, mlkem-ed25519
        #[arg(short, long, default_value = "mlkem-mldsa")]
        algorithm: String,
    },
    /// Sign a file with hybrid signature
    Sign {
        /// File to sign
        file: String,
        /// Private key file
        #[arg(short, long, default_value = "fusion.key.secret")]
        key: String,
    },
    /// Verify a signature
    Verify {
        /// File to verify
        file: String,
        /// Signature file
        sig: String,
        /// Public key file
        #[arg(short, long, default_value = "fusion.key.public")]
        key: String,
    },
    /// Encrypt a file with hybrid PQC
    Encrypt {
        /// File to encrypt
        file: String,
        /// Public key file
        #[arg(short, long, default_value = "fusion.key.public")]
        key: String,
    },
    /// Decrypt a file
    Decrypt {
        /// File to decrypt
        file: String,
        /// Private key file
        #[arg(short, long, default_value = "fusion.key.secret")]
        key: String,
    },
}

#[derive(Subcommand)]
pub enum SourceSubcommand {
    /// List all .fu files in Source Files/
    List,
    /// Compile a Source Files .fu file
    Compile {
        /// File path relative to Source Files/
        file: String,
    },
    /// Run all Source Files tests
    Test {
        /// Filter tests by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Check which files are stubs vs real implementations
    Audit,
}

#[derive(Subcommand)]
pub enum NeuralSealSubcommand {
    /// Generate a NeuralSeal keypair
    Keygen {
        /// Security level: low, medium, or high
        #[arg(short, long, default_value = "medium")]
        level: String,
    },
    /// Encrypt a file with NeuralSeal
    Encrypt {
        /// Path to secret key file
        #[arg(short, long)]
        key: String,
        /// Input file to encrypt
        #[arg(short, long)]
        input: String,
        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decrypt a NeuralSeal encrypted file
    Decrypt {
        /// Path to secret key file
        #[arg(short, long)]
        key: String,
        /// Input file to decrypt
        #[arg(short, long)]
        input: String,
        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Sign a file with NeuralSeal
    Sign {
        /// Path to signing key file
        #[arg(short, long)]
        key: String,
        /// Input file to sign
        #[arg(short, long)]
        input: String,
    },
    /// Verify a NeuralSeal signature
    Verify {
        /// Path to verification key file
        #[arg(short, long)]
        key: String,
        /// Input file to verify
        #[arg(short, long)]
        input: String,
        /// Path to signature file
        #[arg(short, long)]
        signature: String,
    },
}

#[derive(Subcommand)]
pub enum ProviderSubcommand {
    /// List all 26 AI model providers
    List,
    /// Check provider status and configuration
    Status {
        /// Provider ID (e.g. ollama, openai, anthropic)
        provider: String,
    },
    /// Pull a model via Ollama
    Pull {
        /// Model name (e.g. llama3, mistral)
        model: String,
    },
    /// Test model inference
    Test {
        /// Model name to test
        model: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a config value by section.key path
    Get {
        /// Section.key path (e.g. build.optimization_level)
        key: String,
    },
    /// Set a config value by section.key path
    Set {
        /// Section.key path (e.g. build.optimization_level)
        key: String,
        /// Value to set
        value: String,
    },
    /// List all config sections
    List,
    /// Validate the Fusion.toml configuration
    Validate,
    /// Display full Fusion.toml config
    Show,
}
