use clap::Parser;
use colored::*;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "This tool can foreach dirs and run command.",
    long_about
)]
struct Args {
    #[arg(short, long)]
    dir: String,

    #[arg(short, long)]
    cmd: String,

    #[arg(short, long, default_value_t = 1)]
    layers: u8,
}

fn main() {
    let args = Args::parse();
    WalkDir::new(args.dir)
        .max_depth(args.layers.into())
        .follow_links(true)
        .into_iter()
        .filter_map(|result| result.ok())
        .into_iter()
        .filter(|file| file.file_type().is_dir())
        .for_each(|dir| {
            let work_dir = dir.path().to_str().unwrap();
            let mut cmd_collect: Vec<&str> = args.cmd.split(" ").collect();
            let cmd_program = cmd_collect.remove(0);

            println!(
                "{} {} -> {} {}",
                "[RUN COMMAND]".on_bright_blue(),
                work_dir,
                cmd_program.bright_blue(),
                cmd_collect.join(" ").yellow()
            );

            let output = Command::new(cmd_program)
                .args(cmd_collect)
                .current_dir(work_dir)
                .spawn()
                .unwrap()
                .wait_with_output()
                .expect("failed to execute process");
            println!("{}", format!("{:?}", output).purple());
        });
}
