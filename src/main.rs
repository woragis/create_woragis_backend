use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "create_woragis_api")]
#[command(version = "1.0")]
#[command(about = "CLI to scaffold a Rust backend", long_about = None)]
struct Cli {
    /// The project name
    name: String,

    /// Optional template type (rest, grpc, ai-rest, ai-grpc)
    #[arg(short, long, default_value = "rest")]
    template: String,

    /// Include Github Actions CI configuration
    #[arg(long)]
    with_ci: bool,

    /// Include Terraform Infrastructure setup
    #[arg(long)]
    with_infra: bool,
}

fn main() {
    let mut args = Cli::parse();

    // If --with-infra is passed, automatically enable --with-ci
    if args.with_infra {
        args.with_ci = true;
    }

    let project_dir = Path::new(&args.name);

    if project_dir.exists() {
        eprintln!("Directory '{}' already exists!", args.name);
        std::process::exit(1);
    }

    // Create project directory
    fs::create_dir(&project_dir).expect("Failed to create project directory");

    // Copy template
    let template_path = format!("templates/{}", args.template);
    copy_dir_all(&template_path, &project_dir).expect("Failed to copy template");

    // Optional: copy .github/ if --with-ci (including if with_infra is enabled)
    if args.with_ci {
        let ci_template = "extras/.github";
        copy_dir_all(ci_template, &project_dir.join(".github")).expect("Failed to copy CI configs");
    }

    // Optional: copy terraform/ if --with-infra
    if args.with_infra {
        let infra_template = "extras/terraform";
        copy_dir_all(infra_template, &project_dir.join("terraform")).expect("Failed to copy infra setup");
    }

    println!("✅ Project '{}' created using '{}' template.", args.name, args.template);
    if args.with_ci {
        println!("✅ Included GitHub CI (.github/)");
    }
    if args.with_infra {
        println!("✅ Included Terraform (terraform/)");
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &str, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?; // ✅ Ensure the destination directory exists
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(path.to_str().unwrap(), &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?; // ✅ Create parent directories for files
            }
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
