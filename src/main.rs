use clap::Parser;
use colored::*;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    author,
    version = "0.1.1",
    about = "This tool can foreach dirs and run command.",
    long_about
)]
struct Args {
    #[arg(short, long, default_value = "./")]
    dir: String,

    #[arg(short, long)]
    cmd: String,

    #[arg(short, long, default_value_t = 1)]
    layers: u8,

    #[arg(short, long)]
    r#async: bool,
}

fn do_command4_dir_async(work_dir: String, cmd_program: String, cmd_args: Vec<String>) {
    tokio::spawn(async { do_command4_dir(work_dir, cmd_program, cmd_args) });
}

fn do_command4_dir(work_dir: String, cmd_program: String, cmd_args: Vec<String>) {
    let output = Command::new(cmd_program)
        .args(cmd_args)
        .current_dir(work_dir)
        .spawn()
        .unwrap()
        .wait_with_output()
        .expect("failed to execute process");
    println!("{}", format!("{:?}", output).purple());
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    WalkDir::new(args.dir)
        .max_depth(args.layers.into())
        .follow_links(true)
        .into_iter()
        .filter_map(|result| result.ok())
        .into_iter()
        .filter(|file| file.file_type().is_dir())
        .for_each(|dir| {
            let work_dir: String = dir.path().to_str().unwrap().to_string();
            let mut cmd_args: Vec<String> =
                args.cmd.split(' ').map(|arg| arg.to_string()).collect();
            let cmd_program: String = cmd_args.remove(0).to_string();
            println!(
                "{} {} -> {} {}",
                "[RUN COMMAND]".on_bright_blue(),
                work_dir,
                cmd_program.bright_blue(),
                cmd_args.join(" ").yellow()
            );
            match args.r#async {
                true => do_command4_dir_async(work_dir, cmd_program, cmd_args),
                false => do_command4_dir(work_dir, cmd_program, cmd_args),
            }
        });
}
