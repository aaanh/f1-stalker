use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

pub struct InstanceServer {
    focus_rx: mpsc::Receiver<()>,
    _thread: thread::JoinHandle<()>,
}

const FOCUS_TOKEN: &[u8] = b"focus";

pub fn request_focus_if_running() -> bool {
    #[cfg(unix)]
    {
        request_focus_unix()
    }
    #[cfg(not(unix))]
    {
        request_focus_tcp()
    }
}

pub fn start_server() -> Option<InstanceServer> {
    #[cfg(unix)]
    {
        start_server_unix()
    }
    #[cfg(not(unix))]
    {
        start_server_tcp()
    }
}

#[cfg(unix)]
fn socket_path() -> Option<std::path::PathBuf> {
    crate::db::default_db_path()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.join("instance.sock")))
}

#[cfg(unix)]
fn request_focus_unix() -> bool {
    use std::os::unix::net::UnixStream;

    let Some(path) = socket_path() else {
        return false;
    };

    let Ok(mut stream) = UnixStream::connect(&path) else {
        return false;
    };

    let _ = stream.write_all(FOCUS_TOKEN);
    true
}

#[cfg(unix)]
fn start_server_unix() -> Option<InstanceServer> {
    use std::os::unix::net::UnixListener;

    let path = socket_path()?;
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }

    let listener = UnixListener::bind(&path).ok()?;
    let (focus_tx, focus_rx) = mpsc::channel();

    let thread = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buffer = [0_u8; FOCUS_TOKEN.len()];
            if stream.read_exact(&mut buffer).is_ok() && buffer == FOCUS_TOKEN {
                let _ = focus_tx.send(());
            }
        }
    });

    Some(InstanceServer {
        focus_rx,
        _thread: thread,
    })
}

#[cfg(not(unix))]
const INSTANCE_PORT: u16 = 38_421;

#[cfg(not(unix))]
fn request_focus_tcp() -> bool {
    use std::net::TcpStream;

    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", INSTANCE_PORT)) else {
        return false;
    };

    let _ = stream.write_all(FOCUS_TOKEN);
    true
}

#[cfg(not(unix))]
fn start_server_tcp() -> Option<InstanceServer> {
    use std::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", INSTANCE_PORT)).ok()?;
    let (focus_tx, focus_rx) = mpsc::channel();

    let thread = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buffer = [0_u8; FOCUS_TOKEN.len()];
            if stream.read_exact(&mut buffer).is_ok() && buffer == FOCUS_TOKEN {
                let _ = focus_tx.send(());
            }
        }
    });

    Some(InstanceServer {
        focus_rx,
        _thread: thread,
    })
}

impl InstanceServer {
    pub fn poll_focus(&self) -> bool {
        self.focus_rx.try_recv().is_ok()
    }
}
