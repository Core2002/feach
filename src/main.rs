use clap::Parser;
use colored::*;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    author,
    version = "0.1.2",
    about = "This tool can foreach dirs and run command.",
    long_about
)]
struct Args {
    #[arg(short, long, default_value = "./")]
    dir: String,

    #[arg(short, long, num_args = 1..)]
    cmd: Vec<String>,

    #[arg(short, long, default_value_t = 1)]
    layers: u8,

    #[arg(short, long)]
    r#async: bool,
}

fn parse_cmd_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in cmd.chars() {
        match c {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            ' ' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Run the command in one directory. Returns true on success (exit code 0).
fn do_command4_dir(work_dir: String, cmd_program: String, cmd_args: Vec<String>) -> bool {
    let output = match Command::new(&cmd_program)
        .args(&cmd_args)
        .current_dir(&work_dir)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            println!(
                "{} {} -> {}: {}",
                "[FAILED]".on_red(),
                work_dir,
                cmd_program.bright_blue(),
                e
            );
            return false;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("{} {}", work_dir.bright_black(), stdout.trim_end());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        println!("{} {}", work_dir.bright_black(), stderr.trim_end().red());
    }
    if output.status.success() {
        true
    } else {
        println!(
            "{} {} -> exit code {}",
            "[FAILED]".on_red(),
            work_dir,
            output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_string())
        );
        false
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let cmd_str = args.cmd.join(" ");
    let cmd_args = parse_cmd_args(&cmd_str);
    if cmd_args.is_empty() {
        eprintln!("error: empty command");
        std::process::exit(2);
    }
    let cmd_program: String = cmd_args[0].clone();
    let cmd_rest: Vec<String> = cmd_args[1..].to_vec();

    let mut handles = Vec::new();
    let mut failures: usize = 0;
    let mut total: usize = 0;

    for entry in WalkDir::new(&args.dir)
        .max_depth(args.layers.into())
        .follow_links(true)
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|file| file.file_type().is_dir())
    {
        let work_dir: String = entry.path().to_string_lossy().to_string();
        total += 1;
        println!(
            "{} {} -> {} {}",
            "[RUN COMMAND]".on_bright_blue(),
            work_dir,
            cmd_program.bright_blue(),
            cmd_rest.join(" ").yellow()
        );
        if args.r#async {
            let program = cmd_program.clone();
            let rest = cmd_rest.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                do_command4_dir(work_dir, program, rest)
            }));
        } else if !do_command4_dir(work_dir, cmd_program.clone(), cmd_rest.clone()) {
            failures += 1;
        }
    }

    // Wait for every spawned task: without this, the runtime shuts down when
    // main returns and kills commands that are still running.
    for handle in handles {
        match handle.await {
            Ok(true) => {}
            Ok(false) => failures += 1,
            Err(_) => failures += 1,
        }
    }

    if failures > 0 {
        eprintln!(
            "{}",
            format!("[DONE] {}/{} succeeded, {} failed", total - failures, total, failures).red()
        );
        std::process::exit(1);
    }
    println!(
        "{}",
        format!("[DONE] {}/{} succeeded", total, total).green()
    );
}
