use interprocess::unnamed_pipe::pipe;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    os::windows::io::{AsHandle, AsRawHandle},
    process::{Child, Command},
    sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    time::Duration,
};

type WriteRequest = (Vec<u8>, SyncSender<io::Result<()>>);

pub(crate) struct Pipe {
    commands: SyncSender<WriteRequest>,
    messages: Receiver<io::Result<Vec<u8>>>,
}

impl Pipe {
    pub(crate) fn launch(command: &mut Command) -> io::Result<(Child, Self)> {
        let (writer, child_read) = pipe()?;
        let (child_write, reader) = pipe()?;
        // Clones are non-inheritable; only Chrome's ends must survive process creation.
        let mut parent_writer = File::from(writer.as_handle().try_clone_to_owned()?);
        let parent_reader = File::from(reader.as_handle().try_clone_to_owned()?);
        drop((writer, reader));
        command.arg("--remote-debugging-pipe").arg(format!(
            "--remote-debugging-io-pipes={},{}",
            child_read.as_raw_handle() as usize,
            child_write.as_raw_handle() as usize
        ));

        // Blocking Windows pipe I/O runs off-thread so the event pump keeps its deadlines.
        let (commands, pending) = mpsc::sync_channel::<WriteRequest>(1);
        let (received, messages) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("browsershot-pipe-writer".into())
            .spawn(move || {
                while let Ok((message, completed)) = pending.recv() {
                    let result = parent_writer.write_all(&message);
                    let failed = result.is_err();
                    let _ = completed.send(result);
                    if failed {
                        break;
                    }
                }
            })?;
        std::thread::Builder::new()
            .name("browsershot-pipe-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(parent_reader);
                loop {
                    let mut message = Vec::new();
                    let result = reader.read_until(0, &mut message).and_then(|length| {
                        if length == 0 {
                            Err(io::ErrorKind::UnexpectedEof.into())
                        } else {
                            message.pop();
                            Ok(message)
                        }
                    });
                    let failed = result.is_err();
                    if received.send(result).is_err() || failed {
                        break;
                    }
                }
            })?;
        let child = command.spawn()?;
        drop((child_read, child_write));
        Ok((child, Self { commands, messages }))
    }

    pub(crate) fn send(&self, text: &str) -> io::Result<()> {
        let (completed, result) = mpsc::sync_channel(1);
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        self.commands
            .send((bytes, completed))
            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?;
        result
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
    }

    pub(crate) fn receive(&self) -> io::Result<Option<Vec<u8>>> {
        match self.messages.recv_timeout(Duration::from_millis(50)) {
            Ok(message) => message.map(Some),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(io::ErrorKind::BrokenPipe.into()),
        }
    }
}
