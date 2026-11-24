use crate::{ChromiumResult, Error};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
#[cfg(unix)]
use std::io::{Read, Write};
use std::{
    collections::{HashMap, HashSet},
    net::{TcpStream, ToSocketAddrs},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

enum Transport {
    Starting,
    WebSocket(Box<WebSocket<MaybeTlsStream<TcpStream>>>),
    #[cfg(windows)]
    WindowsPipe(crate::windows_pipe::Pipe),
    #[cfg(unix)]
    Pipe {
        reader: std::os::unix::net::UnixStream,
        writer: std::os::unix::net::UnixStream,
        buffer: Vec<u8>,
    },
}

pub(crate) struct Connection {
    transport: Transport,
    child: Option<Child>,
    _profile: Option<tempfile::TempDir>,
    target: Option<String>,
    session: Option<String>,
    next_id: u64,
    deadline: Instant,
    protocol_timeout: Duration,
    pub(crate) result: ChromiumResult,
    options: Value,
    pub(crate) loaded: bool,
    pub(crate) dom_loaded: bool,
    pub(crate) inflight: HashSet<String>,
    pub(crate) status: Option<u16>,
    main_frame: Option<String>,
    requests: HashMap<String, String>,
    auth_attempts: HashSet<String>,
    initial_content: Option<(String, String)>,
    initial_file: Option<String>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Give Chrome time to flush persistent profiles before forcing termination.
            if self.send("Browser.close", json!({}), false).is_ok() {
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(2) {
                    match child.try_wait() {
                        Ok(Some(_)) => return,
                        Err(_) => break,
                        Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                    }
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        } else if let Some(target) = self.target.take() {
            // Only close our target, never the remote user's browser.
            let _ = self.send("Target.closeTarget", json!({"targetId":target}), false);
        }
    }
}

impl Connection {
    pub(crate) fn open(shot: &crate::Browsershot) -> Result<Self, Error> {
        let deadline = Instant::now() + shot.duration("timeout");
        let remote = shot.options["browserWSEndpoint"]
            .as_str()
            .map(str::to_owned)
            .map(Ok)
            .or_else(|| {
                shot.options["remoteInstanceUrl"]
                    .as_str()
                    .map(|url| remote_endpoint(url, deadline))
            });
        let mut connection = if let Some(endpoint) = remote {
            match endpoint.and_then(|endpoint| Self::websocket(&endpoint, deadline)) {
                Ok(transport) => Self::new(transport, None, None, shot, deadline),
                Err(error) if shot.options["throwOnRemoteConnectionError"] == true => {
                    return Err(Error::RemoteConnection(error.to_string()));
                }
                Err(_) => Self::launch(shot, deadline)?,
            }
        } else {
            Self::launch(shot, deadline)?
        };
        let target = connection
            .call_browser("Target.createTarget", json!({"url":"about:blank"}))?["targetId"]
            .as_str()
            .ok_or_else(|| Error::Protocol("missing target id".into()))?
            .to_owned();
        connection.target = Some(target.clone());
        let session = connection.call_browser(
            "Target.attachToTarget",
            json!({"targetId":target,"flatten":true}),
        )?["sessionId"]
            .as_str()
            .ok_or_else(|| Error::Protocol("missing session id".into()))?
            .to_owned();
        connection.session = Some(session);
        Ok(connection)
    }

    fn new(
        transport: Transport,
        child: Option<Child>,
        profile: Option<tempfile::TempDir>,
        shot: &crate::Browsershot,
        deadline: Instant,
    ) -> Self {
        Self {
            transport,
            child,
            _profile: profile,
            target: None,
            session: None,
            next_id: 0,
            deadline,
            protocol_timeout: shot.duration("protocolTimeout"),
            result: ChromiumResult::default(),
            options: shot.options.clone(),
            loaded: false,
            dom_loaded: false,
            inflight: HashSet::new(),
            status: None,
            main_frame: None,
            requests: HashMap::new(),
            auth_attempts: HashSet::new(),
            initial_content: None,
            initial_file: None,
        }
    }

    fn launch(shot: &crate::Browsershot, deadline: Instant) -> Result<Self, Error> {
        let executable = shot.options["executablePath"]
            .as_str()
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::var_os("CHROME_PATH").map(Into::into))
            .or_else(find_chrome)
            .ok_or(Error::ChromeNotFound)?;
        let profile = if let Some(path) = &shot.temp_path {
            tempfile::tempdir_in(path)?
        } else {
            tempfile::tempdir()?
        };
        let profile_path = shot.options["userDataDir"]
            .as_str()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| profile.path().into());
        if !profile_path.is_absolute() {
            return Err(Error::InvalidInput(
                "user data directory must be absolute".into(),
            ));
        }
        let mut command = Command::new(executable);
        command
            .args([
                "--headless=new",
                "--disable-gpu",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-dev-shm-usage",
                "--hide-scrollbars",
                "--allow-file-access-from-files",
            ])
            .arg(format!("--user-data-dir={}", profile_path.display()))
            .args(&shot.arguments)
            .envs(&shot.environment)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if shot.options["noSandbox"] == true {
            command.arg("--no-sandbox");
        }
        if let Some(proxy) = shot.options["proxyServer"].as_str() {
            command.arg(format!("--proxy-server={proxy}"));
        }
        if shot.options["pipe"] == true {
            #[cfg(unix)]
            {
                use command_fds::{CommandFdExt, FdMapping};
                use std::os::{fd::OwnedFd, unix::net::UnixStream};
                let (writer, child_read) = UnixStream::pair()?;
                let (reader, child_write) = UnixStream::pair()?;
                reader.set_read_timeout(Some(Duration::from_millis(50)))?;
                writer.set_write_timeout(Some(Duration::from_secs(1)))?;
                command.arg("--remote-debugging-pipe");
                command
                    .fd_mappings(vec![
                        FdMapping {
                            parent_fd: OwnedFd::from(child_read),
                            child_fd: 3,
                        },
                        FdMapping {
                            parent_fd: OwnedFd::from(child_write),
                            child_fd: 4,
                        },
                    ])
                    .map_err(|error| Error::Protocol(error.to_string()))?;
                let child = command.spawn()?;
                return Ok(Self::new(
                    Transport::Pipe {
                        reader,
                        writer,
                        buffer: Vec::new(),
                    },
                    Some(child),
                    Some(profile),
                    shot,
                    deadline,
                ));
            }
            #[cfg(windows)]
            {
                let (child, pipe) = crate::windows_pipe::Pipe::launch(&mut command)?;
                return Ok(Self::new(
                    Transport::WindowsPipe(pipe),
                    Some(child),
                    Some(profile),
                    shot,
                    deadline,
                ));
            }
            #[cfg(not(any(unix, windows)))]
            return Err(Error::InvalidInput(
                "DevTools pipes are not supported on this platform".into(),
            ));
        }
        command.arg("--remote-debugging-port=0");
        let endpoint_file = profile_path.join("DevToolsActivePort");
        let previous_endpoint = std::fs::read_to_string(&endpoint_file).ok();
        let child = command.spawn()?;
        // Install the child guard before any fallible startup work.
        let mut guard = Self::new(
            Transport::Starting,
            Some(child),
            Some(profile),
            shot,
            deadline,
        );
        loop {
            guard.check_deadline()?;
            if let Some(status) = guard
                .child
                .as_mut()
                .map(Child::try_wait)
                .transpose()?
                .flatten()
            {
                return Err(Error::ChromeExited(status.to_string()));
            }
            if let Ok(contents) = std::fs::read_to_string(&endpoint_file)
                && Some(&contents) != previous_endpoint.as_ref()
            {
                let mut lines = contents.lines();
                if let (Some(port), Some(path)) = (lines.next(), lines.next()) {
                    guard.transport =
                        Self::websocket(&format!("ws://127.0.0.1:{port}{path}"), deadline)?;
                    return Ok(guard);
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn websocket(endpoint: &str, deadline: Instant) -> Result<Transport, Error> {
        let url = url::Url::parse(endpoint)
            .map_err(|error| Error::RemoteConnection(error.to_string()))?;
        if !matches!(url.scheme(), "ws" | "wss") {
            return Err(Error::InvalidInput(
                "WebSocket endpoint must use ws or wss".into(),
            ));
        }
        let host = url
            .host_str()
            .ok_or_else(|| Error::InvalidInput("missing WebSocket hostname".into()))?;
        let port = url
            .port_or_known_default()
            .ok_or_else(|| Error::InvalidInput("missing WebSocket port".into()))?;
        let mut last_error = None;
        for address in (host, port).to_socket_addrs()? {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(Error::Timeout)?;
            match TcpStream::connect_timeout(&address, remaining.min(Duration::from_secs(5))) {
                Ok(stream) => {
                    stream.set_read_timeout(Some(remaining))?;
                    stream.set_write_timeout(Some(remaining))?;
                    let (mut socket, _) = tungstenite::client_tls(endpoint, stream)
                        .map_err(|error| Error::RemoteConnection(error.to_string()))?;
                    match socket.get_mut() {
                        MaybeTlsStream::Plain(stream) => {
                            stream.set_read_timeout(Some(Duration::from_millis(50)))?;
                            stream.set_write_timeout(Some(Duration::from_secs(1)))?;
                        }
                        MaybeTlsStream::Rustls(stream) => {
                            stream
                                .sock
                                .set_read_timeout(Some(Duration::from_millis(50)))?;
                            stream
                                .sock
                                .set_write_timeout(Some(Duration::from_secs(1)))?;
                        }
                        _ => return Err(Error::Protocol("unsupported TLS transport".into())),
                    }
                    return Ok(Transport::WebSocket(Box::new(socket)));
                }
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error
            .map(Error::Io)
            .unwrap_or_else(|| Error::RemoteConnection("hostname has no addresses".into())))
    }

    pub(crate) fn check_deadline(&self) -> Result<(), Error> {
        if Instant::now() >= self.deadline {
            Err(Error::Timeout)
        } else {
            Ok(())
        }
    }

    fn send(&mut self, method: &str, params: Value, session: bool) -> Result<u64, Error> {
        self.next_id += 1;
        let mut message = json!({"id":self.next_id,"method":method,"params":params});
        if session {
            message["sessionId"] = json!(self.session);
        }
        let text = message.to_string();
        match &mut self.transport {
            Transport::Starting => return Err(Error::Protocol("Chrome is still starting".into())),
            Transport::WebSocket(socket) => socket.send(Message::Text(text.into()))?,
            #[cfg(windows)]
            Transport::WindowsPipe(pipe) => pipe.send(&text)?,
            #[cfg(unix)]
            Transport::Pipe { writer, .. } => {
                writer.write_all(text.as_bytes())?;
                writer.write_all(&[0])?;
            }
        }
        Ok(self.next_id)
    }

    fn receive(&mut self) -> Result<Option<Value>, Error> {
        self.check_deadline()?;
        match &mut self.transport {
            Transport::Starting => Err(Error::Protocol("Chrome is still starting".into())),
            #[cfg(windows)]
            Transport::WindowsPipe(pipe) => pipe
                .receive()?
                .map(|bytes| serde_json::from_slice(&bytes).map_err(Error::from))
                .transpose(),
            Transport::WebSocket(socket) => match socket.read() {
                Ok(Message::Text(text)) => Ok(Some(serde_json::from_str(&text)?)),
                Ok(Message::Close(_)) => {
                    Err(Error::Protocol("Chrome closed the connection".into()))
                }
                Ok(_) => Ok(None),
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    Ok(None)
                }
                Err(error) => Err(error.into()),
            },
            #[cfg(unix)]
            Transport::Pipe { reader, buffer, .. } => {
                if let Some(end) = buffer.iter().position(|byte| *byte == 0) {
                    let message = serde_json::from_slice(&buffer[..end])?;
                    buffer.drain(..=end);
                    return Ok(Some(message));
                }
                let mut chunk = [0; 65536];
                match reader.read(&mut chunk) {
                    Ok(0) => Err(Error::Protocol("Chrome closed the pipe".into())),
                    Ok(count) => {
                        buffer.extend_from_slice(&chunk[..count]);
                        Ok(None)
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) =>
                    {
                        Ok(None)
                    }
                    Err(error) => Err(error.into()),
                }
            }
        }
    }

    fn call_browser(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.call_inner(method, params, false)
    }
    pub(crate) fn call(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.call_inner(method, params, true)
    }
    fn call_inner(&mut self, method: &str, params: Value, session: bool) -> Result<Value, Error> {
        let deadline = (Instant::now() + self.protocol_timeout).min(self.deadline);
        let id = self.send(method, params, session)?;
        loop {
            if Instant::now() >= deadline {
                return Err(Error::Timeout);
            }
            if let Some(message) = self.receive()? {
                if message["id"] == id {
                    if !message["error"].is_null() {
                        return Err(Error::Protocol(format!("{method}: {}", message["error"])));
                    }
                    return Ok(message["result"].clone());
                }
                self.event(&message)?;
            }
        }
    }

    pub(crate) fn pump(&mut self) -> Result<(), Error> {
        if let Some(message) = self.receive()? {
            self.event(&message)?;
        }
        Ok(())
    }

    pub(crate) fn sleep(&mut self, duration: Duration) -> Result<(), Error> {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            self.pump()?;
        }
        Ok(())
    }

    pub(crate) fn content(&mut self, url: String, html: String) {
        self.initial_content = Some((url, html));
    }
    pub(crate) fn initial_file(&mut self, url: String) {
        self.initial_file = Some(url);
    }
    pub(crate) fn block_local_files(&mut self) {
        self.options["blockLocalFileAccess"] = json!(true);
    }

    pub(crate) fn main_frame(&mut self, frame: String) {
        self.main_frame = Some(frame);
    }

    fn response(&mut self, response: &Value, main: bool) {
        let record = json!({"url":response["url"],"status":response["status"],"reason":response["statusText"],"headers":response["headers"]});
        if main {
            self.status = response["status"].as_f64().map(|value| value as u16);
            self.result.redirect_history.push(record);
        }
        if response["status"]
            .as_f64()
            .is_some_and(|status| !(200.0..400.0).contains(&status))
        {
            self.result
                .failed_requests
                .push(json!({"status":response["status"],"url":response["url"]}));
        }
    }

    fn event(&mut self, event: &Value) -> Result<(), Error> {
        if event["sessionId"].as_str() != self.session.as_deref() {
            return Ok(());
        }
        let params = &event["params"];
        match event["method"].as_str().unwrap_or("") {
            "Page.frameNavigated" if params["frame"]["parentId"].is_null() => {
                self.main_frame = params["frame"]["id"].as_str().map(str::to_owned)
            }
            "Page.loadEventFired" => self.loaded = true,
            "Page.domContentEventFired" => self.dom_loaded = true,
            "Page.javascriptDialogOpening" if self.options["dismissDialogs"] == true => {
                self.send("Page.handleJavaScriptDialog", json!({"accept":false}), true)?;
            }
            "Network.requestWillBeSent" => {
                let id = params["requestId"].as_str().unwrap_or("").to_owned();
                let url = params["request"]["url"].as_str().unwrap_or("").to_owned();
                self.inflight.insert(id.clone());
                self.requests.insert(id, url.clone());
                if self.options["disableCaptureURLs"] != true {
                    self.result.requests.push(json!({"url":url}));
                }
                if !params["redirectResponse"].is_null() {
                    self.response(
                        &params["redirectResponse"],
                        params["type"] == "Document"
                            && params["frameId"].as_str() == self.main_frame.as_deref(),
                    );
                }
            }
            "Network.loadingFinished" | "Network.loadingFailed" => {
                let id = params["requestId"].as_str().unwrap_or("");
                self.inflight.remove(id);
                if event["method"] == "Network.loadingFailed" {
                    self.result
                        .failed_requests
                        .push(json!({"url":self.requests.get(id),"error":params["errorText"]}));
                }
            }
            "Network.responseReceived" => self.response(
                &params["response"],
                params["type"] == "Document"
                    && params["frameId"].as_str() == self.main_frame.as_deref(),
            ),
            "Runtime.consoleAPICalled" => {
                let text = params["args"]
                    .as_array()
                    .map(|args| {
                        args.iter()
                            .map(|arg| {
                                arg["value"].as_str().map(str::to_owned).unwrap_or_else(|| {
                                    if arg["value"].is_null() {
                                        arg["description"]
                                            .as_str()
                                            .unwrap_or("undefined")
                                            .to_owned()
                                    } else {
                                        arg["value"].to_string()
                                    }
                                })
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                let frame = &params["stackTrace"]["callFrames"][0];
                self.result.console_messages.push(json!({"type":params["type"],"message":text,"location":{"url":frame["url"],"lineNumber":frame["lineNumber"],"columnNumber":frame["columnNumber"]},"stackTrace":params["stackTrace"]["callFrames"]}));
            }
            "Runtime.exceptionThrown" => {
                let details = &params["exceptionDetails"];
                self.result.page_errors.push(json!({"name":details["exception"]["className"].as_str().unwrap_or("Error"),"message":details["exception"]["description"].as_str().or(details["text"].as_str()).unwrap_or("unknown error")}));
            }
            "Fetch.authRequired" => {
                let id = params["requestId"].as_str().unwrap_or("").to_owned();
                let first = self.auth_attempts.insert(id.clone());
                self.send("Fetch.continueWithAuth",json!({"requestId":id,"authChallengeResponse":{"response":if first {"ProvideCredentials"} else {"CancelAuth"},"username":self.options["authentication"]["username"],"password":self.options["authentication"]["password"]}}),true)?;
            }
            "Fetch.requestPaused" => self.intercept(params)?,
            _ => {}
        }
        Ok(())
    }

    fn intercept(&mut self, params: &Value) -> Result<(), Error> {
        let request = &params["request"];
        let url = request["url"].as_str().unwrap_or("");
        let document = params["resourceType"] == "Document";
        let local = url.starts_with("file:");
        let initial = self.initial_file.as_deref() == Some(url) && document;
        if initial {
            self.initial_file = None;
        }
        let blocked = (local
            && self.options["blockLocalFileAccess"] == true
            && self.options["allowLocalFileAccess"] != true
            && !initial)
            || (self.options["disableImages"] == true && params["resourceType"] == "Image")
            || self.options["blockUrls"].as_array().is_some_and(|items| {
                items
                    .iter()
                    .any(|item| item.as_str().is_some_and(|part| url.contains(part)))
            })
            || self.options["blockDomains"]
                .as_array()
                .is_some_and(|items| {
                    url::Url::parse(url)
                        .ok()
                        .is_some_and(|url| items.iter().any(|item| item.as_str() == url.host_str()))
                })
            || (self.options["disableRedirects"] == true
                && !params["redirectedRequestId"].is_null());
        if blocked {
            self.send(
                "Fetch.failRequest",
                json!({"requestId":params["requestId"],"errorReason":"BlockedByClient"}),
                true,
            )?;
            return Ok(());
        }
        let mut headers = request["headers"].as_object().cloned().unwrap_or_default();
        if document && let Some(extra) = self.options["extraNavigationHTTPHeaders"].as_object() {
            headers.extend(extra.clone());
        }
        if let Some((content_url, html)) = &self.initial_content
            && url.trim_end_matches('/') == content_url.trim_end_matches('/')
            && document
        {
            self.send("Fetch.fulfillRequest",json!({"requestId":params["requestId"],"responseCode":200,"responseHeaders":[{"name":"Content-Type","value":"text/html; charset=utf-8"}],"body":STANDARD.encode(html)}),true)?;
            return Ok(());
        }
        let mut continuation = json!({"requestId":params["requestId"]});
        if let Some(fields) = self.options["postParams"].as_object()
            && document
        {
            let data = fields
                .iter()
                .map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("&");
            continuation["method"] = json!("POST");
            continuation["postData"] = json!(STANDARD.encode(data));
            headers.insert(
                "Content-Type".into(),
                json!("application/x-www-form-urlencoded"),
            );
        }
        continuation["headers"] = json!(
            headers
                .into_iter()
                .map(|(name, value)| json!({"name":name,"value":value}))
                .collect::<Vec<_>>()
        );
        self.send("Fetch.continueRequest", continuation, true)?;
        Ok(())
    }
}

fn remote_endpoint(url: &str, deadline: Instant) -> Result<String, Error> {
    let timeout = deadline
        .checked_duration_since(Instant::now())
        .ok_or(Error::Timeout)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout.min(Duration::from_secs(5))))
        .build()
        .into();
    let response: Value = agent
        .get(format!("{}/json/version", url.trim_end_matches('/')))
        .call()
        .map_err(|error| Error::RemoteConnection(error.to_string()))?
        .body_mut()
        .read_json()
        .map_err(|error| Error::RemoteConnection(error.to_string()))?;
    response["webSocketDebuggerUrl"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::RemoteConnection("missing webSocketDebuggerUrl".into()))
}

fn find_chrome() -> Option<std::path::PathBuf> {
    let names = [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome.exe",
    ];
    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths) {
            for name in names {
                let candidate = directory.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    for path in [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    ] {
        if std::path::Path::new(path).is_file() {
            return Some(path.into());
        }
    }
    None
}
