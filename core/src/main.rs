use gamezoea::app::window;
use gamezoea::emu::gb::*;

use std::{env, fs, process, thread};

const DEFAULT_SCALE: u32 = 1;

fn main() {
    let (scale, rom, steps) = parse_args();

    let rom_path = match rom {
        Some(rom) => {
            eprintln!("Opening rom {:?}", rom.display());
            rom
        }
        None => {
            eprintln!("No rom specified! Use --rom <file>.gb");
            return;
        }
    };

    // Validate that the ROM has a .gb extension
    if !rom_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("gb"))
        .unwrap_or(false)
    {
        eprintln!(
            "Error: ROM file must have a .gb extension, got: {:?}",
            rom_path.display()
        );
        return;
    }

    let rom_bytes = match fs::read(&rom_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Failed to read rom {:?}: {err}", rom_path.display());
            return;
        }
    };

    if rom_bytes.is_empty() {
        eprintln!("Rom {:?} is empty", rom_path.display());
        return;
    }

    let rom_data = rom_bytes.into_boxed_slice();

    if scale == 0 {
        run_headless(rom_data, steps);
        return;
    }

    run_windowed(rom_data, scale);
}

fn parse_args() -> (u32, Option<std::path::PathBuf>, Option<u64>) {
    let mut args = env::args();
    let _ = args.next();

    let mut scale = DEFAULT_SCALE;
    let mut path = None;
    let mut steps = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scale" | "-s" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("Missing value for {arg}");
                    usage();
                    process::exit(1);
                });

                scale = match value.parse::<u32>() {
                    Ok(0) => 0,
                    Ok(scale) if (1..=window::MAX_SCALE).contains(&scale) => scale,
                    Err(_) => {
                        eprintln!("Invalid scale value: {value}");
                        usage();
                        process::exit(1);
                    }
                    Ok(scale) => {
                        eprintln!(
                            "Scale {scale} is outside the supported range 0..={}",
                            window::MAX_SCALE
                        );
                        usage();
                        process::exit(1);
                    }
                };
            }

            "--steps" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("Missing value for {arg}");
                    usage();
                    process::exit(1);
                });

                steps = match value.parse::<u64>() {
                    Ok(0) => None, // 0 means run forever
                    Ok(n) => Some(n),
                    Err(_) => {
                        eprintln!("Invalid steps value: {value}");
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

    (scale, path, steps)
}

fn usage() {
    println!(
        "Usage: gamezoea [--scale <0 (headless) or 1..={}>]",
        window::MAX_SCALE
    );
    println!("                [--rom <rom.gb>]");
    println!("                [--steps <number of CPU cycles to run, 0 or omitted = run forever>]");
}

fn run_headless(rom_data: Box<[u8]>, steps: Option<u64>) {
    let gameboy_thread = thread::spawn(move || {
        let mut gameboy = Gameboy::headless_dmg(&rom_data);
        match steps {
            Some(n) => {
                for _ in 0..n {
                    gameboy.step(1);
                }
            }
            None => gameboy.run(),
        }
    });

    gameboy_thread.join().unwrap();
}

fn run_windowed(rom_data: Box<[u8]>, scale: u32) {
    let mut threads = vec![];
    let (frame_tx, frame_rx) = window::create_frame_channel();

    let window_thread = thread::spawn(move || {
        if let Err(err) = window::run(scale, frame_rx) {
            eprintln!("Window error: {err}");
        }
    });
    threads.push(window_thread);

    let gameboy_thread = thread::spawn(move || {
        let mut gameboy = Gameboy::dmg(&rom_data, frame_tx);
        gameboy.run();
    });
    threads.push(gameboy_thread);

    for thread in threads {
        thread.join().unwrap();
    }
}
