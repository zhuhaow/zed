use std::{path::Path, sync::Arc, thread::JoinHandle};

use anyhow::Context;
use clap::Parser;
use cli::{ipc::IpcOneShotServer, CliRequest, CliResponse, IpcHandshake};
use parking_lot::Mutex;
use release_channel::ReleaseChannel;
use util::ResultExt;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, GENERIC_WRITE, HANDLE},
        Storage::FileSystem::{
            CreateFileW, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE,
            OPEN_EXISTING, PIPE_ACCESS_INBOUND,
        },
        System::{
            Pipes::{
                ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
                PIPE_TYPE_MESSAGE, PIPE_WAIT,
            },
            Threading::CreateMutexW,
        },
    },
};

use crate::{Args, OpenListener};

#[inline]
fn retrieve_app_identifier() -> &'static str {
    match *release_channel::RELEASE_CHANNEL {
        ReleaseChannel::Dev => "Zed-Editor-Dev",
        ReleaseChannel::Nightly => "Zed-Editor-Nightly",
        ReleaseChannel::Preview => "Zed-Editor-Preview",
        ReleaseChannel::Stable => "Zed-Editor-Stable",
    }
}

#[inline]
fn generate_identifier(name: &str) -> HSTRING {
    HSTRING::from(format!("{}-{}", retrieve_app_identifier(), name))
}

#[inline]
fn generate_identifier_with_prefix(prefix: &str, name: &str) -> HSTRING {
    HSTRING::from(format!("{}{}-{}", prefix, retrieve_app_identifier(), name))
}

pub fn check_single_instance(opener: OpenListener) -> bool {
    unsafe {
        CreateMutexW(None, false, &generate_identifier("Instance-Mutex"))
            .expect("Unable to create instance sync mutex")
    };
    let last_err = unsafe { GetLastError() };
    let ret = last_err != ERROR_ALREADY_EXISTS;

    if ret {
        // We are the first instance
        std::thread::spawn(move || {
            with_pipe(|url| opener.open_urls(vec![url]));
        });
    } else {
        // We are not the first instance
        send_args_to_instance().log_err();
    }

    ret
}

fn with_pipe(f: impl Fn(String)) {
    let pipe = unsafe {
        CreateNamedPipeW(
            &generate_identifier_with_prefix("\\\\.\\pipe\\", "Named-Pipe"),
            PIPE_ACCESS_INBOUND,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            1,
            128,
            128,
            0,
            None,
        )
    };
    if pipe.is_invalid() {
        log::error!("Failed to create named pipe: {:?}", unsafe {
            GetLastError()
        });
        return;
    }

    loop {
        if let Some(message) = retrieve_message_from_pipe(pipe)
            .context("Failed to read from named pipe")
            .log_err()
        {
            f(message);
        }
    }
}

// This part of code is mostly from crates/cli/src/main.rs
fn send_args_to_instance() -> anyhow::Result<()> {
    let args = Args::parse();
    let (server, server_name) =
        IpcOneShotServer::<IpcHandshake>::new().context("Handshake before Zed spawn")?;
    let url = format!("zed-cli://{server_name}");

    let exit_status = Arc::new(Mutex::new(None));
    let mut paths = vec![];
    let mut urls = vec![];
    for path in args.paths_or_urls.iter() {
        if path.starts_with("zed://")
            || path.starts_with("http://")
            || path.starts_with("https://")
            || path.starts_with("file://")
            || path.starts_with("ssh://")
        {
            urls.push(path.to_string());
        } else if let Some(path) = std::fs::canonicalize(Path::new(path)).ok() {
            paths.push(path.to_string_lossy().to_string());
        }
    }

    let sender: JoinHandle<anyhow::Result<()>> = std::thread::spawn({
        let exit_status = exit_status.clone();
        move || {
            let (_, handshake) = server.accept().context("Handshake after Zed spawn")?;
            let (tx, rx) = (handshake.requests, handshake.responses);

            tx.send(CliRequest::Open {
                paths,
                urls,
                wait: false,
                open_new_workspace: None,
                env: None,
            })?;

            while let Ok(response) = rx.recv() {
                match response {
                    CliResponse::Ping => {}
                    CliResponse::Stdout { message } => log::info!("{message}"),
                    CliResponse::Stderr { message } => log::error!("{message}"),
                    CliResponse::Exit { status } => {
                        exit_status.lock().replace(status);
                        return Ok(());
                    }
                }
            }

            Ok(())
        }
    });

    unsafe {
        let pipe = CreateFileW(
            &generate_identifier_with_prefix("\\\\.\\pipe\\", "Named-Pipe"),
            GENERIC_WRITE.0,
            FILE_SHARE_MODE::default(),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES::default(),
            None,
        )?;
        let message = url.as_bytes();
        let mut bytes_written = 0;
        WriteFile(pipe, Some(message), Some(&mut bytes_written), None)?;
        CloseHandle(pipe)?;
    }
    sender.join().unwrap()?;
    if let Some(exit_status) = exit_status.lock().take() {
        std::process::exit(exit_status);
    }
    Ok(())
}

fn retrieve_message_from_pipe(pipe: HANDLE) -> anyhow::Result<String> {
    unsafe { ConnectNamedPipe(pipe, None)? };
    let message = retrieve_message_from_pipe_inner(pipe);
    unsafe { DisconnectNamedPipe(pipe).log_err() };
    message
}

fn retrieve_message_from_pipe_inner(pipe: HANDLE) -> anyhow::Result<String> {
    let mut buffer = [0u8; 128];
    unsafe {
        ReadFile(pipe, Some(&mut buffer), None, None)?;
    }
    let message = std::ffi::CStr::from_bytes_until_nul(&buffer)?;
    Ok(message.to_string_lossy().to_string())
}
