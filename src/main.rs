use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

mod mapper;
mod uinput_controller;

use arboard::Clipboard;
use uinput_controller::UinputController;

fn socket_path() -> PathBuf {
    let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime).join("keycast.sock")
}

enum Cmd {
    Fix,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--trigger" {
        send_trigger();
    } else {
        run_daemon();
    }
}

fn run_daemon() {
    let sock = socket_path();

    if sock.exists() {
        if UnixStream::connect(&sock).is_ok() {
            eprintln!("keycast: already running");
            return;
        }
        fs::remove_file(&sock).expect("Failed to remove stale socket");
    }

    let (sender, receiver) = mpsc::sync_channel::<Cmd>(1);

    thread::spawn(move || {
        let mut uinput = match UinputController::new() {
            Ok(dev) => dev,
            Err(err) => {
                eprintln!("Failed to create virtual keyboard: {err}");
                eprintln!("Are you in the 'input' group? (sudo usermod -aG input $USER)");
                std::process::exit(1);
            }
        };
        eprintln!("keycast: virtual keyboard ready");

        let mut clipboard = match Clipboard::new() {
            Ok(clip) => clip,
            Err(err) => {
                eprintln!("Failed to access clipboard: {err}");
                std::process::exit(1);
            }
        };
        eprintln!("keycast: clipboard ready");

        while let Ok(Cmd::Fix) = receiver.recv() {
            loop {
                match receiver.try_recv() {
                    Ok(Cmd::Fix) => continue,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            fix_selection(&mut uinput, &mut clipboard);
        }
    });

    let listener = UnixListener::bind(&sock).expect("Failed to bind Unix socket");
    eprintln!("keycast: listening on {:?}", sock);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 1];
                if stream.read_exact(&mut buf).is_ok() && buf[0] == 0x01 {
                    let _ = sender.try_send(Cmd::Fix);
                }
            }
            Err(err) => eprintln!("Connection failed: {err}"),
        }
    }
}

fn send_trigger() {
    let sock = socket_path();

    match UnixStream::connect(&sock) {
        Ok(mut stream) => {
            if stream.write_all(&[0x01]).is_err() {
                eprintln!("Failed to send trigger to daemon.");
            }
            let _ = stream.flush();
        }
        Err(_) => eprintln!("Error: keycast daemon is not running"),
    }
}

fn fix_selection(uinput: &mut UinputController, clipboard: &mut Clipboard) {
    eprintln!("keycast: fix triggered");

    uinput.simulate_ctrl_c();
    thread::sleep(Duration::from_millis(100));

    let text = match clipboard.get_text() {
        Ok(content) => content,
        Err(err) => {
            eprintln!("keycast: failed to read clipboard: {err}");
            return;
        }
    };

    if text.is_empty() {
        eprintln!("keycast: clipboard is empty");
        return;
    }

    let direction = mapper::LayoutMapper::detect_direction(&text);
    eprintln!("keycast: direction = {:?}", direction);

    let fixed = mapper::LayoutMapper::translate(&text, direction);

    if fixed == text {
        eprintln!("keycast: text unchanged, skipping");
        return;
    }

    eprintln!("keycast: \"{}\" > \"{}\"", text, fixed);

    match clipboard.set_text(&fixed) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("keycast: failed to write clipboard: {err}");
            return;
        }
    }

    thread::sleep(Duration::from_millis(50));
    uinput.simulate_ctrl_v();
}
