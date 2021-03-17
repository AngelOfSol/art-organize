use std::{
    fmt::Debug,
    io::{Read, Write},
};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use tokio::{sync::mpsc, task};

use serde::{Deserialize, Serialize};

const ADDRESS: &str = "ao_ipc";

fn handle_connection<T: Send + Sync + 'static + for<'de> Deserialize<'de> + Debug>(
    tx: mpsc::Sender<T>,
    mut incoming: LocalSocketStream,
) -> anyhow::Result<()> {
    let mut buffer = vec![0; 1000];
    let size = incoming.read(&mut buffer)?;
    buffer.truncate(size);

    let message = bincode::deserialize(&buffer)?;

    tx.blocking_send(message)?;

    Ok(())
}

pub type IpcReceiver<T> = mpsc::Receiver<T>;

pub fn start_server<T: Send + Sync + 'static + for<'de> Deserialize<'de> + Debug>(
) -> anyhow::Result<IpcReceiver<T>> {
    let socket = LocalSocketListener::bind(ADDRESS)?;

    let (tx, rx) = mpsc::channel(4);

    task::spawn_blocking(move || {
        for incoming in socket.incoming().filter_map(Result::ok) {
            let tx = tx.clone();
            let _ = handle_connection(tx, incoming);
        }
    });

    Ok(rx)
}

pub struct IpcSender {
    stream: LocalSocketStream,
}

impl IpcSender {
    pub fn send<T: Serialize>(mut self, value: T) -> anyhow::Result<()> {
        self.stream.write_all(&bincode::serialize(&value)?)?;
        self.stream.flush()?;
        Ok(())
    }
}

pub fn try_connect() -> Option<IpcSender> {
    let stream = LocalSocketStream::connect(ADDRESS).ok()?;
    Some(IpcSender { stream })
}
