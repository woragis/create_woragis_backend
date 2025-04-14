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
}

fn main() {
    let args = Cli::parse();
    let project_dir = Path::new(&args.name);

    if project_dir.exists() {
        eprintln!("Directory '{}' already exists!", args.name);
        std::process::exit(1);
    }

    // Create project directory
    fs::create_dir(&project_dir).expect("Failed to create project directory");

    // Copy template (we’ll make this in the next step)
    let template_path = format!("templates/{}/base", args.template);
    copy_dir_all(&template_path, &project_dir).expect("Failed to copy template");

    println!("✅ Project '{}' created using '{}' template.", args.name, args.template);
}

/// Recursively copy a directory
fn copy_dir_all(src: &str, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_all(path.to_str().unwrap(), &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
