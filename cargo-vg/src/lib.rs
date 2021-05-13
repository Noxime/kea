use std::{
    path::{Path, PathBuf},
    rc::Rc,
};
use structopt::StructOpt;
use vg_native::runtime::wasm::Wasm;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Args {
    #[structopt(name = "vg")]
    Vg(Opts),
}

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,
    #[structopt()]
    pub build_path: Option<PathBuf>,
    #[structopt(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum Cmd {
    /// Build the project and launch it
    Run,
    /// Run the project on every file change
    Watch,
    /// Build the project
    Build,
    /// Clean the project build files
    Clean,
}

fn run_cargo(manifest: impl AsRef<Path>, build: Option<PathBuf>, step: impl AsRef<str>) -> bool {
    let mut cmd = std::process::Command::new("cargo");

    cmd.arg(step.as_ref())
        .arg("--manifest-path")
        .arg(manifest.as_ref())
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    if let Some(path) = build {
        cmd.arg("--target-dir").arg(path);
    }

    cmd.status().unwrap().success()
}

fn run_engine(idle_task: impl Fn() -> bool + 'static) {
    let wasm = std::fs::read("target/wasm32-unknown-unknown/debug/rust-test.wasm").unwrap();
    vg_native::Engine::run::<Wasm, _>(&wasm, idle_task)
}

pub fn run(opts: Opts) {
    // let existing = std::env::var("RUSTFLAGS").unwrap_or_default();
    // std::env::set_var("RUSTFLAGS", "-C link-arg=--import-memory");

    match opts.cmd {
        None | Some(Cmd::Run) => {
            if run_cargo(&opts.manifest_path, opts.build_path, "build") {
                println!("Running project");
                run_engine(|| true);
            }
        }
        Some(Cmd::Watch) => {
            println!("Watching project for changes");

            use notify::{watcher, RecursiveMode, Watcher};
            use std::sync::mpsc::channel;
            use std::time::Duration;

            let (tx, rx) = channel();
            let rx = Rc::new(rx);

            let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

            watcher
                .watch(&opts.manifest_path, RecursiveMode::Recursive)
                .unwrap();
            watcher
                .watch(
                    &opts.manifest_path.with_file_name("src"),
                    RecursiveMode::Recursive,
                )
                .unwrap();

            loop {
                let local_rx = Rc::clone(&rx);
                if run_cargo(&opts.manifest_path, opts.build_path.clone(), "build") {
                    println!("Running project");
                    run_engine(move || {
                        matches!(
                            local_rx.try_recv(),
                            Err(std::sync::mpsc::TryRecvError::Empty)
                        )
                    });
                }

                println!("Reloading")
            }
        }
        Some(Cmd::Build) => {
            println!("Building project");
            run_cargo(opts.manifest_path, opts.build_path, "build");
        }
        Some(Cmd::Clean) => {
            println!("Cleaning project");
            run_cargo(opts.manifest_path, opts.build_path, "clean");
        }
    }
}