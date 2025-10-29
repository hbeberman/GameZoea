#![allow(dead_code)]
use gamezoea::app::window;
use gamezoea::emu::cpu::*;
use std::{env, process, thread};

const DEFAULT_SCALE: u32 = 1;

fn main() {
    let mut threads = vec![];
    let (scale, rom) = parse_args();

    let cpu = Cpu::default();
    println!("GameZoea!");
    println!("{}", cpu);
    match &rom {
        Some(rom) => eprintln!("Opening rom {:?}", rom.display()),
        None => {
            eprintln!("No rom specified! Use --rom <file>.gb");
            return;
        }
    }

    let shared_pixels = window::create_pixels_handle();

    let window_pixels = shared_pixels.clone();

    let window_thread = thread::spawn(move || {
        if let Err(err) = window::run(scale, window_pixels) {
            eprintln!("Window error: {err}");
        }
    });
    threads.push(window_thread);

    let gameboy_thread = thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    threads.push(gameboy_thread);

    for thread in threads {
        thread.join().unwrap();
    }
}

fn parse_args() -> (u32, Option<std::path::PathBuf>) {
    let mut args = env::args();
    let _ = args.next();

    let mut scale = DEFAULT_SCALE;
    let mut path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scale" | "-s" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("Missing value for {arg}");
                    usage();
                    process::exit(1);
                });

                scale = match value.parse::<u32>() {
                    Ok(0) => 1,
                    Ok(scale) => scale,
                    Err(_) => {
                        eprintln!("Invalid scale value: {value}");
                        usage();
                        process::exit(1);
                    }
                };
            }

            "--rom" | "-r" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("Missing value for {arg}");
                    usage();
                    process::exit(1);
                });

                if value.starts_with("--") {
                    eprintln!("Missing value for {arg}");
                    usage();
                    process::exit(1);
                }

                let tpath = std::path::Path::new(&value);
                path = Some(if tpath.is_absolute() {
                    tpath.to_path_buf()
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => cwd.join(tpath),
                        Err(_) => tpath.to_path_buf(),
                    }
                });
            }

            "--help" | "-h" => {
                usage();
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                usage();
                process::exit(1);
            }
        }
    }

    (scale, path)
}

fn usage() {
    println!("Usage: gamezoea [--scale <positive integer>]");
    println!("                [--rom <rom.gb>]");
}
