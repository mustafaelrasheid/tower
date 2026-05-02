use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tower")]
#[command(version = "0.1.0")]
#[command(author = "mustafaelrasheid")]
#[command(
    about = "Package manager for bricks",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    RebuildLock {
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
    Validate {
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
        #[arg(long, default_value = "")]
        root_dir: String,
    },
    Export {
        packages: Vec<String>,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
        #[arg(long, default_value = "")]
        root_dir: String,
    },
    Install {
        packages: Vec<String>,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
        #[arg(long, default_value = "")]
        root_dir: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        yes: bool,
    },
    Purge {
        packages: Vec<String>,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
        #[arg(long, default_value = "")]
        root_dir: String,
        #[arg(long)]
        yes: bool,
    },
    Convert {
        packages: Vec<String>,
        #[arg(long, num_args=1..)]
        deps: Vec<String>,
    },
    GetDeb {
        packages: Vec<String>,
    },
    Tag {
        package: String,
        group: String,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
    Untag {
        package: String,
        group: String,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
    CreateGroup {
        group: String,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
    ListGroup {
        group: String,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
    DeleteGroup {
        group: String,
        #[arg(long, default_value = "/var/lib/tower")]
        lib_dir: String,
    },
} 
