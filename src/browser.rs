use crate::{Browsershot, Error, Source, cdp::Connection};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub(crate) enum Action<'a> {
    Screenshot,
    Pdf,
    Evaluate(&'a str),
    Diagnostics,
}

pub(crate) fn run(shot: &mut Browsershot, action: Action<'_>) -> Result<Value, Error> {
    validate(shot)?;
    let mut connection = Connection::open(shot)?;
    let result = render(&mut connection, shot, action);
    shot.result = Some(std::mem::take(&mut connection.result));
    result
}

fn validate(shot: &Browsershot) -> Result<(), Error> {
    if let Source::Url(url) = &shot.source {
        validate_url(url)?;
    }
    if let Source::Html(html) = &shot.source {
        let decoded = html_escape::decode_html_entities(html).to_ascii_lowercase();
        let local_reference = regex::Regex::new(
            r"(//|\\\\)\s*(localhost[/\\:\s]|127\.|0\.0\.0\.0[/\\:\s]|\[::1\][/\\:\s]|::1[/\\:\s])",
        )
        .map_err(|error| Error::InvalidInput(error.to_string()))?;
        // Explicit HTTP(S) URLs are permitted; only schemeless local references are rejected.
        let local = local_reference.captures_iter(&decoded).any(|capture| {
            let start = capture.get(0).map_or(0, |matched| matched.start());
            &capture[1] != "//" || start == 0 || decoded.as_bytes()[start - 1] != b':'
        });
        if decoded.contains("view-source") || local {
            return Err(Error::InvalidInput(
                "HTML cannot contain view-source URLs or schemeless local network paths".into(),
            ));
        }
    }
    if let Some(url) = shot.options["contentUrl"].as_str() {
        validate_url(url)?;
    }
    let known = [
        "args",
        "env",
        "viewport",
        "timeout",
        "protocolTimeout",
        "fullPage",
        "noSandbox",
        "dismissDialogs",
        "disableJavascript",
        "disableImages",
        "disableCaptureURLS",
        "disableRedirects",
        "acceptInsecureCerts",
        "tagged",
        "omitBackground",
        "pipe",
        "executablePath",
        "proxyServer",
        "userAgent",
        "userDataDir",
        "headerTemplate",
        "footerTemplate",
        "pageRanges",
        "format",
        "browserWSEndpoint",
        "remoteInstanceUrl",
        "throwOnRemoteConnectionError",
        "contentUrl",
        "device",
        "evaluateOnNewDocument",
        "landscape",
        "scale",
        "type",
        "quality",
        "encoding",
        "clip",
        "selector",
        "selectorIndex",
        "width",
        "height",
        "margin",
        "displayHeaderFooter",
        "printBackground",
        "delay",
        "waitUntil",
        "networkIdleTimeout",
        "function",
        "functionPolling",
        "functionTimeout",
        "waitForSelector",
        "waitForSelectorOptions",
        "emulateMedia",
        "emulateMediaFeatures",
        "allowLocalFileAccess",
        "preventUnsuccessfulResponse",
        "authentication",
        "postParams",
        "cookies",
        "extraHTTPHeaders",
        "extraNavigationHTTPHeaders",
        "blockUrls",
        "blockDomains",
        "types",
        "selects",
        "clicks",
        "locatorClicks",
        "initialPageNumber",
        "addStyleTag",
        "addScriptTag",
        "preferCSSPageSize",
        "generateDocumentOutline",
        "captureBeyondViewport",
        "fromSurface",
        "optimizeForSpeed",
        "blockLocalFileAccess",
        "newHeadless",
        "outline",
        "waitForFonts",
        "path",
        "pageFunction",
    ];
    if let Some(options) = shot.options.as_object() {
        for key in options.keys() {
            if !known.contains(&key.as_str()) {
                return Err(Error::InvalidInput(format!("unknown browser option {key}")));
            }
        }
    }
    if shot.options["quality"]
        .as_u64()
        .is_some_and(|quality| quality > 100)
    {
        return Err(Error::InvalidInput(
            "screenshot quality must be 0–100".into(),
        ));
    }
    Ok(())
}

fn validate_url(value: &str) -> Result<(), Error> {
    let url = url::Url::parse(value).map_err(|error| Error::InvalidInput(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(Error::InvalidInput(
            "only HTTP and HTTPS URLs are allowed; use html_from_file_path for trusted files"
                .into(),
        ));
    }
    Ok(())
}

fn render(
    connection: &mut Connection,
    shot: &Browsershot,
    action: Action<'_>,
) -> Result<Value, Error> {
    let options = &shot.options;
    for method in ["Page.enable", "Runtime.enable", "Network.enable"] {
        connection.call(method, json!({}))?;
    }
    let frame_tree = connection.call("Page.getFrameTree", json!({}))?;
    if let Some(frame) = frame_tree["frameTree"]["frame"]["id"].as_str() {
        connection.main_frame(frame.to_owned());
    }
    if let Some(agent) = options["userAgent"].as_str() {
        connection.call("Network.setUserAgentOverride", json!({"userAgent":agent}))?;
    }
    let mut viewport = json!({"width":800,"height":600,"deviceScaleFactor":1,"isMobile":false});
    if let Some(device) = options["device"].as_str() {
        let devices: Value = serde_json::from_str(include_str!("devices.json"))?;
        let device = devices
            .get(device)
            .ok_or_else(|| Error::InvalidInput(format!("unknown device {device}")))?;
        viewport = device["viewport"].clone();
        connection.call(
            "Network.setUserAgentOverride",
            json!({"userAgent":device["userAgent"]}),
        )?;
    }
    if let Some(overrides) = options["viewport"].as_object() {
        for (key, value) in overrides {
            viewport[key] = value.clone();
        }
    }
    connection.call("Emulation.setDeviceMetricsOverride",json!({"width":viewport["width"],"height":viewport["height"],"deviceScaleFactor":viewport["deviceScaleFactor"].as_f64().unwrap_or(1.0),"mobile":viewport["isMobile"]==true,"screenOrientation":{"type":if viewport["isLandscape"]==true {"landscapePrimary"} else {"portraitPrimary"},"angle":if viewport["isLandscape"]==true {90} else {0}}}))?;
    connection.call(
        "Emulation.setTouchEmulationEnabled",
        json!({"enabled":viewport["hasTouch"]==true}),
    )?;
    if options["acceptInsecureCerts"] == true {
        connection.call(
            "Security.setIgnoreCertificateErrors",
            json!({"ignore":true}),
        )?;
    }
    if options["disableJavascript"] == true {
        connection.call(
            "Emulation.setScriptExecutionDisabled",
            json!({"value":true}),
        )?;
    }
    if options["extraHTTPHeaders"].is_object() {
        connection.call(
            "Network.setExtraHTTPHeaders",
            json!({"headers":options["extraHTTPHeaders"]}),
        )?;
    }
    if options["cookies"].is_array() {
        connection.call("Network.setCookies", json!({"cookies":options["cookies"]}))?;
    }
    if let Some(script) = options["evaluateOnNewDocument"].as_str() {
        connection.call(
            "Page.addScriptToEvaluateOnNewDocument",
            json!({"source":expression(script)}),
        )?;
    }
    if !options["emulateMedia"].is_null() || !options["emulateMediaFeatures"].is_null() {
        let features = decode_option(&options["emulateMediaFeatures"])?;
        connection.call("Emulation.setEmulatedMedia",json!({"media":options["emulateMedia"].as_str().unwrap_or(""),"features":features.as_array().cloned().unwrap_or_default()}))?;
    } else if matches!(action, Action::Pdf) {
        connection.call("Emulation.setEmulatedMedia", json!({"media":"print"}))?;
    }

    let mut temp_html = None;
    let url = match &shot.source {
        Source::Url(url) => url.trim().to_owned(),
        Source::File(path) if options["contentUrl"].is_string() => {
            let url = options["contentUrl"].as_str().unwrap_or("").to_owned();
            connection.content(url.clone(), std::fs::read_to_string(path)?);
            url
        }
        Source::File(path) => url::Url::from_file_path(std::fs::canonicalize(path)?)
            .map_err(|()| Error::InvalidInput("invalid HTML file path".into()))?
            .to_string(),
        Source::Html(html) => {
            if let Some(url) = options["contentUrl"].as_str() {
                connection.content(url.to_owned(), html.clone());
                url.to_owned()
            } else {
                let directory = if let Some(path) = &shot.temp_path {
                    tempfile::tempdir_in(path)?
                } else {
                    tempfile::tempdir()?
                };
                let path = directory.path().join("index.html");
                std::fs::write(&path, html)?;
                let url = url::Url::from_file_path(&path)
                    .map_err(|()| Error::InvalidInput("invalid temporary path".into()))?
                    .to_string();
                connection.initial_file(url.clone());
                temp_html = Some(directory);
                url
            }
        }
    };
    let html = matches!(shot.source, Source::Html(_));
    let intercept = html
        || options["contentUrl"].is_string()
        || options["blockLocalFileAccess"] == true
        || options["disableImages"] == true
        || options["disableRedirects"] == true
        || ["blockDomains", "blockUrls"].iter().any(|key| {
            options[key]
                .as_array()
                .is_some_and(|values| !values.is_empty())
        })
        || options["extraNavigationHTTPHeaders"].is_object()
        || options["authentication"].is_object()
        || options["postParams"].is_object();
    if html {
        connection.block_local_files();
    }
    if intercept {
        connection.call("Fetch.enable",json!({"patterns":[{"urlPattern":"*"}],"handleAuthRequests":options["authentication"].is_object()}))?;
    }
    connection.loaded = false;
    connection.dom_loaded = false;
    let navigation = connection.call("Page.navigate", json!({"url":url}))?;
    if let Some(error) = navigation["errorText"].as_str() {
        return Err(Error::Protocol(format!("navigation: {error}")));
    }
    let waits = match &options["waitUntil"] {
        Value::Array(waits) => waits.clone(),
        Value::String(wait) => vec![json!(wait)],
        Value::Null => vec![json!("load")],
        _ => {
            return Err(Error::InvalidInput(
                "waitUntil must be a lifecycle name or array".into(),
            ));
        }
    };
    let idle_timeout = Duration::from_millis(options["networkIdleTimeout"].as_u64().unwrap_or(500));
    let mut idle_since = None;
    loop {
        let mut ready = true;
        for wait in &waits {
            ready &= match wait.as_str().unwrap_or("") {
                "domcontentloaded" => connection.dom_loaded,
                "networkidle" | "networkidle0" | "networkidle2" => {
                    if connection.inflight.len() > if wait == "networkidle2" { 2 } else { 0 } {
                        idle_since = None;
                    } else {
                        idle_since.get_or_insert_with(Instant::now);
                    }
                    connection.dom_loaded
                        && idle_since.is_some_and(|start| start.elapsed() >= idle_timeout)
                }
                "load" => connection.loaded,
                other => {
                    return Err(Error::InvalidInput(format!(
                        "unsupported waitUntil {other}"
                    )));
                }
            };
        }
        if ready {
            break;
        }
        connection.pump()?;
    }
    if options["preventUnsuccessfulResponse"] == true
        && let Some(status) = connection.status.filter(|status| *status >= 400)
    {
        return Err(Error::UnsuccessfulResponse(status));
    }
    if options["disableImages"] == true {
        evaluate(
            connection,
            "document.querySelectorAll('img').forEach(image => image.remove())",
        )?;
    }
    interactions(connection, options)?;
    for (key, tag, attribute) in [
        ("addStyleTag", "style", "href"),
        ("addScriptTag", "script", "src"),
    ] {
        if !options[key].is_null() {
            let mut value = decode_option(&options[key])?;
            if let Some(path) = value["path"].as_str() {
                value["content"] = json!(std::fs::read_to_string(path)?);
            }
            let script = format!(
                "async () => {{ const o={value}; const e=document.createElement(o.url && {tag:?}==='style'?'link':{tag:?}); if(o.type)e.type=o.type; if(o.content)e.textContent=o.content; if(o.url){{e[{attribute:?}]=o.url;if({tag:?}==='style')e.rel='stylesheet';}} const loaded=o.url?new Promise((resolve,reject)=>{{e.onload=resolve;e.onerror=()=>reject(new Error('Could not load '+o.url));}}):Promise.resolve(); document.head.appendChild(e); await loaded; }}"
            );
            evaluate(connection, &script)?;
        }
    }
    if let Some(delay) = options["delay"].as_u64() {
        connection.sleep(Duration::from_millis(delay))?;
    }
    if let Some(count) = options["initialPageNumber"].as_u64() {
        evaluate(
            connection,
            &format!(
                "() => {{ const style=document.createElement('style');style.textContent='.empty-page {{page-break-after:always;visibility:hidden}}';document.head.appendChild(style);window.pageStart={count};document.body.prepend(...Array.from({{length:{count}}},()=>{{const e=document.createElement('div');e.className='empty-page';e.textContent='empty';return e;}})); }}"
            ),
        )?;
    }
    if let Some(function) = options["function"].as_str() {
        wait_function(
            connection,
            function,
            &options["functionPolling"],
            options["functionTimeout"]
                .as_u64()
                .filter(|timeout| *timeout > 0)
                .unwrap_or_else(|| options["timeout"].as_u64().unwrap_or(60000)),
        )?;
    }
    if let Some(selector) = options["waitForSelector"].as_str() {
        wait_selector(
            connection,
            selector,
            &options["waitForSelectorOptions"],
            options["timeout"].as_u64().unwrap_or(60000),
        )?;
    }
    let output = match action {
        Action::Evaluate(script) => evaluate(
            connection,
            options["pageFunction"].as_str().unwrap_or(script),
        ),
        Action::Diagnostics => Ok(Value::Null),
        Action::Screenshot => screenshot(connection, options),
        Action::Pdf => pdf(connection, options),
    };
    drop(temp_html);
    if let Some(path) = options["path"].as_str()
        && matches!(action, Action::Screenshot | Action::Pdf)
        && let Ok(value) = &output
    {
        std::fs::write(
            path,
            STANDARD.decode(
                value
                    .as_str()
                    .ok_or_else(|| Error::Protocol("missing binary output".into()))?,
            )?,
        )?;
    }
    output
}

fn decode_option(value: &Value) -> Result<Value, Error> {
    if let Some(text) = value.as_str() {
        Ok(serde_json::from_str(text)?)
    } else {
        Ok(value.clone())
    }
}

fn expression(script: &str) -> String {
    let trimmed = script.trim();
    if trimmed.starts_with("function")
        || trimmed.starts_with("async ")
        || trimmed.starts_with("()")
        || (trimmed.starts_with('(')
            && trimmed
                .find("=>")
                .is_some_and(|arrow| !trimmed[..arrow].contains(';')))
    {
        format!("({script})()")
    } else {
        script.to_owned()
    }
}

fn evaluate(connection: &mut Connection, script: &str) -> Result<Value, Error> {
    let response=connection.call("Runtime.evaluate",json!({"expression":expression(script),"awaitPromise":true,"returnByValue":true,"userGesture":true}))?;
    if !response["exceptionDetails"].is_null() {
        return Err(Error::Protocol(format!(
            "JavaScript: {}",
            response["exceptionDetails"]
        )));
    }
    if !response["result"]["unserializableValue"].is_null() {
        return Err(Error::Protocol(format!(
            "JavaScript value cannot be represented as JSON: {}",
            response["result"]["unserializableValue"]
        )));
    }
    Ok(response["result"]["value"].clone())
}

fn wait_function(
    connection: &mut Connection,
    function: &str,
    polling: &Value,
    timeout: u64,
) -> Result<(), Error> {
    let polling = if polling.is_null() {
        json!("raf")
    } else {
        polling.clone()
    };
    let condition = expression(function);
    let script = format!(
        "() => new Promise((resolve,reject)=>{{const timer=setTimeout(()=>{{observer?.disconnect();reject(new Error('waitForFunction timed out'));}},{timeout});let observer;const test=async()=>{{try{{if(await ({condition})){{clearTimeout(timer);observer?.disconnect();resolve(true);return true;}}}}catch(error){{clearTimeout(timer);observer?.disconnect();reject(error);return true;}}return false;}};if({polling}==='mutation'){{observer=new MutationObserver(test);observer.observe(document,{{childList:true,subtree:true,attributes:true,characterData:true}});test();}}else{{const tick=async()=>{{if(!await test()){{if({polling}==='raf')requestAnimationFrame(tick);else setTimeout(tick,{polling});}}}};tick();}}}})"
    );
    match evaluate(connection, &script) {
        Err(Error::Protocol(message)) if message.contains("waitForFunction timed out") => {
            return Err(Error::Timeout);
        }
        result => {
            result?;
        }
    }
    Ok(())
}

fn wait_selector(
    connection: &mut Connection,
    selector: &str,
    options: &Value,
    timeout: u64,
) -> Result<(), Error> {
    let selector = serde_json::to_string(selector)?;
    let visible = options["visible"] == true;
    let hidden = options["hidden"] == true;
    let condition = format!(
        "() => {{ const e=document.querySelector({selector}); const r=e?.getBoundingClientRect(); const visible=!!e && getComputedStyle(e).visibility!=='hidden' && getComputedStyle(e).visibility!=='collapse' && r.width>0 && r.height>0; return {} ; }}",
        if hidden {
            "!visible"
        } else if visible {
            "visible"
        } else {
            "!!e"
        }
    );
    wait_function(
        connection,
        &condition,
        &json!("raf"),
        options["timeout"]
            .as_u64()
            .filter(|value| *value > 0)
            .unwrap_or(timeout),
    )
}

fn interactions(connection: &mut Connection, options: &Value) -> Result<(), Error> {
    if let Some(types) = options["types"].as_array() {
        for entry in types {
            let selector = entry["selector"].to_string();
            if evaluate(
                connection,
                &format!(
                    "() => {{const e=document.querySelector({selector});if(!e)return false;e.focus();return true;}}"
                ),
            )? != true
            {
                return Err(Error::ElementNotFound(selector));
            }
            let text = entry["text"].as_str().unwrap_or("");
            let delay = entry["delay"].as_u64().unwrap_or(0);
            for character in text.chars() {
                let text = character.to_string();
                if character.is_ascii() {
                    let key = if character == '\n' { "Enter" } else { &text };
                    let code = match character {
                        'a'..='z' | 'A'..='Z' => format!("Key{}", character.to_ascii_uppercase()),
                        '0'..='9' => format!("Digit{character}"),
                        '\n' => "Enter".into(),
                        ' ' => "Space".into(),
                        _ => String::new(),
                    };
                    connection.call("Input.dispatchKeyEvent", json!({"type":"keyDown","key":key,"code":code,"text":if character == '\n' { "\r" } else { &text },"unmodifiedText":text,"windowsVirtualKeyCode":character.to_ascii_uppercase() as u32}))?;
                    connection.sleep(Duration::from_millis(delay))?;
                    connection.call("Input.dispatchKeyEvent", json!({"type":"keyUp","key":key,"code":code,"windowsVirtualKeyCode":character.to_ascii_uppercase() as u32}))?;
                } else {
                    connection.call("Input.insertText", json!({"text":text}))?;
                    connection.sleep(Duration::from_millis(delay))?;
                }
            }
        }
    }
    if let Some(selects) = options["selects"].as_array() {
        for entry in selects {
            let selector = entry["selector"].to_string();
            let value = entry["value"].to_string();
            if evaluate(
                connection,
                &format!(
                    "() => {{ const e=document.querySelector({selector});if(!e)return false;if(e.tagName!=='SELECT')throw new Error('Element is not a select');for(const option of e.options)option.selected=option.value==={value};e.dispatchEvent(new Event('input',{{bubbles:true}}));e.dispatchEvent(new Event('change',{{bubbles:true}}));return true; }}"
                ),
            )? != true
            {
                return Err(Error::ElementNotFound(selector));
            }
        }
    }
    for key in ["clicks", "locatorClicks"] {
        if let Some(clicks) = options[key].as_array() {
            for entry in clicks {
                let selector = entry["selector"].as_str().unwrap_or("");
                if key == "locatorClicks" {
                    wait_selector(
                        connection,
                        selector,
                        &json!({"visible":true}),
                        options["timeout"].as_u64().unwrap_or(60000),
                    )?;
                    wait_function(
                        connection,
                        &format!(
                            "async () => {{const e=document.querySelector({});if(!e || e.matches(':disabled'))return false;const a=e.getBoundingClientRect();await new Promise(requestAnimationFrame);const b=e.getBoundingClientRect();return a.x===b.x && a.y===b.y && a.width===b.width && a.height===b.height;}}",
                            serde_json::to_string(selector)?
                        ),
                        &json!("raf"),
                        options["timeout"].as_u64().unwrap_or(60000),
                    )?;
                }
                let bounds = bounds(connection, selector, 0)?;
                let x = bounds["x"].as_f64().unwrap_or(0.0)
                    + bounds["width"].as_f64().unwrap_or(0.0) / 2.0;
                let y = bounds["y"].as_f64().unwrap_or(0.0)
                    + bounds["height"].as_f64().unwrap_or(0.0) / 2.0;
                connection.call(
                    "Input.dispatchMouseEvent",
                    json!({"type":"mouseMoved","x":x,"y":y}),
                )?;
                for count in 1..=entry["clickCount"].as_u64().unwrap_or(1) {
                    for event in ["mousePressed", "mouseReleased"] {
                        connection.call("Input.dispatchMouseEvent",json!({"type":event,"x":x,"y":y,"button":entry["button"],"clickCount":count}))?;
                        if event == "mousePressed" {
                            connection.sleep(Duration::from_millis(
                                entry["delay"].as_u64().unwrap_or(0),
                            ))?;
                        }
                    }
                }
                connection.call("Page.getFrameTree", json!({}))?;
                while !connection.loaded {
                    connection.pump()?;
                }
            }
        }
    }
    Ok(())
}

fn bounds(connection: &mut Connection, selector: &str, index: u64) -> Result<Value, Error> {
    let quoted = serde_json::to_string(selector)?;
    let value = evaluate(
        connection,
        &format!(
            "() => {{ const e=document.querySelectorAll({quoted})[{index}]; if(!e)return null;e.scrollIntoView({{block:'center',inline:'center',behavior:'instant'}});const r=e.getBoundingClientRect();return r.width && r.height ? {{x:r.x,y:r.y,width:r.width,height:r.height,scale:1}}:null; }}"
        ),
    )?;
    if value.is_null() {
        Err(Error::ElementNotFound(selector.into()))
    } else {
        Ok(value)
    }
}

fn screenshot(connection: &mut Connection, options: &Value) -> Result<Value, Error> {
    let mut params = json!({
        "format": options["type"].as_str().unwrap_or("png"),
        "captureBeyondViewport": options["fullPage"] == true
            || options["selector"].is_string()
            || options["clip"].is_object(),
    });
    for key in [
        "quality",
        "fromSurface",
        "captureBeyondViewport",
        "optimizeForSpeed",
    ] {
        if !options[key].is_null() {
            params[key] = options[key].clone();
        }
    }
    if options["omitBackground"] == true {
        connection.call(
            "Emulation.setDefaultBackgroundColorOverride",
            json!({"color":{"r":0,"g":0,"b":0,"a":0}}),
        )?;
    }
    if let Some(selector) = options["selector"].as_str() {
        let mut clip = bounds(
            connection,
            selector,
            options["selectorIndex"].as_u64().unwrap_or(0),
        )?;
        let scroll = evaluate(connection, "({x:window.scrollX,y:window.scrollY})")?;
        clip["x"] = json!(clip["x"].as_f64().unwrap_or(0.0) + scroll["x"].as_f64().unwrap_or(0.0));
        clip["y"] = json!(clip["y"].as_f64().unwrap_or(0.0) + scroll["y"].as_f64().unwrap_or(0.0));
        params["clip"] = clip;
    } else if options["clip"].is_object() {
        params["clip"] = options["clip"].clone();
        if params["clip"]["scale"].is_null() {
            params["clip"]["scale"] = json!(1);
        }
    } else if options["fullPage"] == true {
        let metrics = connection.call("Page.getLayoutMetrics", json!({}))?;
        params["clip"] = json!({"x":0,"y":0,"width":metrics["cssContentSize"]["width"],"height":metrics["cssContentSize"]["height"],"scale":1});
    }
    Ok(connection.call("Page.captureScreenshot", params)?["data"].clone())
}

fn inches(value: &Value) -> Result<f64, Error> {
    if let Some(number) = value.as_f64() {
        return Ok(number / 96.0);
    }
    let text = value.as_str().ok_or_else(|| {
        Error::InvalidInput("paper dimensions must be a number or CSS length".into())
    })?;
    let (number, divisor) = if let Some(number) = text.strip_suffix("mm") {
        (number, 25.4)
    } else if let Some(number) = text.strip_suffix("cm") {
        (number, 2.54)
    } else if let Some(number) = text.strip_suffix("in") {
        (number, 1.0)
    } else if let Some(number) = text.strip_suffix("px") {
        (number, 96.0)
    } else {
        (text, 96.0)
    };
    number
        .trim()
        .parse::<f64>()
        .map(|number| number / divisor)
        .map_err(|error| Error::InvalidInput(error.to_string()))
}

fn pdf(connection: &mut Connection, options: &Value) -> Result<Value, Error> {
    let mut params = json!({"printBackground":false,"generateTaggedPDF":false,"marginTop":0,"marginRight":0,"marginBottom":0,"marginLeft":0});
    for key in [
        "landscape",
        "displayHeaderFooter",
        "headerTemplate",
        "footerTemplate",
        "printBackground",
        "scale",
        "pageRanges",
        "preferCSSPageSize",
        "generateDocumentOutline",
    ] {
        if !options[key].is_null() {
            params[key] = options[key].clone();
        }
    }
    params["generateTaggedPDF"] = json!(
        options["tagged"] != false
            || options["outline"] == true
            || options["generateDocumentOutline"] == true
    );
    if let Some(format) = options["format"].as_str() {
        let (width, height) = match format.to_ascii_lowercase().as_str() {
            "letter" => (8.5, 11.0),
            "legal" => (8.5, 14.0),
            "tabloid" => (11.0, 17.0),
            "ledger" => (17.0, 11.0),
            "a0" => (33.1, 46.8),
            "a1" => (23.4, 33.1),
            "a2" => (16.54, 23.4),
            "a3" => (11.7, 16.54),
            "a4" => (8.27, 11.7),
            "a5" => (5.83, 8.27),
            "a6" => (4.13, 5.83),
            _ => {
                return Err(Error::InvalidInput(format!(
                    "unknown paper format {format}"
                )));
            }
        };
        params["paperWidth"] = json!(width);
        params["paperHeight"] = json!(height);
    } else {
        for (key, target) in [("width", "paperWidth"), ("height", "paperHeight")] {
            if !options[key].is_null() {
                params[target] = json!(inches(&options[key])?);
            }
        }
    }
    for (key, target) in [
        ("top", "marginTop"),
        ("right", "marginRight"),
        ("bottom", "marginBottom"),
        ("left", "marginLeft"),
    ] {
        if !options["margin"][key].is_null() {
            params[target] = json!(inches(&options["margin"][key])?);
        }
    }
    if options["omitBackground"] == true {
        connection.call(
            "Emulation.setDefaultBackgroundColorOverride",
            json!({"color":{"r":0,"g":0,"b":0,"a":0}}),
        )?;
    }
    if !options["outline"].is_null() {
        params["generateDocumentOutline"] = options["outline"].clone();
    }
    if options["waitForFonts"] != false {
        evaluate(connection, "async () => { await document.fonts.ready; }")?;
    }
    Ok(connection.call("Page.printToPDF", params)?["data"].clone())
}
