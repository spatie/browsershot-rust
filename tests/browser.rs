//! Public browser behavior, exercised against an external Chrome executable.

use serde_json::json;
use spatie_browsershot::{Browsershot, ClickOptions, Error, Polling, Unit, WaitForSelectorOptions};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

fn html(content: &str) -> Browsershot {
    Browsershot::html(content).no_sandbox().timeout(15)
}

struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    fn start() -> Self {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let url = format!("http://{}", server.server_addr());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = std::thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                let Some(mut request) = server.recv_timeout(Duration::from_millis(20)).unwrap()
                else {
                    continue;
                };
                let path = request.url().to_owned();
                let response=match path.as_str() {
                    path if path.starts_with("http://proxy-fixture.invalid/") && !request.headers().iter().any(|header|header.field.equiv("Proxy-Authorization") && header.value.as_str()=="Basic dXNlcjpwYXNz")=>tiny_http::Response::from_string("proxy authentication").with_status_code(407).with_header(tiny_http::Header::from_bytes("Proxy-Authenticate","Basic realm=\"proxy\"").unwrap()),
                    path if path.starts_with("http://proxy-fixture.invalid/")=>tiny_http::Response::from_string("<h1>Authenticated proxy</h1>"),
                    "/redirect"=>tiny_http::Response::from_string("").with_status_code(302).with_header(tiny_http::Header::from_bytes("Location","/final").unwrap()),
                    "/final"=>tiny_http::Response::from_string("<h1>Final</h1>"),
                    "/missing"=>tiny_http::Response::from_string("missing").with_status_code(404),
                    "/auth" if !request.headers().iter().any(|header|header.field.equiv("Authorization") && header.value.as_str()=="Basic dXNlcjpwYXNz")=>tiny_http::Response::from_string("authenticate").with_status_code(401).with_header(tiny_http::Header::from_bytes("WWW-Authenticate","Basic realm=\"test\"").unwrap()),
                    "/auth"=>tiny_http::Response::from_string("authenticated"),
                    "/inspect"=>{
                        let mut body=String::new();request.as_reader().read_to_string(&mut body).unwrap();
                        let headers=request.headers().iter().map(|header|(header.field.to_string().to_lowercase(),header.value.to_string())).collect::<std::collections::BTreeMap<_,_>>();
                        tiny_http::Response::from_string(format!("<pre>{}</pre>",json!({"method":request.method().as_str(),"body":body,"headers":headers})))
                    }
                    "/asset.js"=>tiny_http::Response::from_string("window.assetLoaded=true;"),
                    "/"=>tiny_http::Response::from_string("<h1>Fixture</h1><script src='/asset.js'></script><img src='/missing'><script>console.log('hello',42);setTimeout(()=>{throw new Error('broken fixture')},10)</script>"),
                    _=>tiny_http::Response::from_string("<h1>Other</h1>"),
                }.with_header(tiny_http::Header::from_bytes("Content-Type","text/html; charset=utf-8").unwrap());
                let _ = request.respond(response);
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
        }
    }
    fn shot(&self, path: &str) -> Browsershot {
        Browsershot::url(format!("{}{path}", self.url))
            .no_sandbox()
            .timeout(15)
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}

#[test]
fn evaluates_scripts_and_keeps_diagnostics_from_the_same_operation() {
    let mut shot = html(
        "<!DOCTYPE html><h1>Original</h1><script>document.querySelector('h1').textContent='Rendered';console.log('ready');throw new Error('example')</script>",
    );
    assert_eq!(
        shot.evaluate("document.querySelector('h1').textContent")
            .unwrap(),
        "Rendered"
    );
    assert_eq!(shot.console_messages().unwrap()[0]["message"], "ready");
    assert!(
        shot.page_errors().unwrap()[0]["message"]
            .as_str()
            .unwrap()
            .contains("example")
    );
    assert_eq!(shot.triggered_requests().unwrap().len(), 1);
    let document = shot.body_html().unwrap();
    assert!(document.starts_with("<!DOCTYPE html>\n<html>"));
    assert!(document.contains("<h1>Rendered</h1>"));
}

#[test]
fn screenshots_have_requested_pixels_crop_density_and_transparency() {
    let content = "<style>body{margin:0}div{width:137px;height:53px;background:rgb(210,30,80)}div+div{width:71px;height:29px;background:rgb(20,180,60)}</style><div></div><div></div>";
    let bytes = html(content)
        .window_size(320, 200)
        .device_scale_factor(2)
        .screenshot()
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (640, 400));
    assert_eq!(image.get_pixel(20, 20).0, [210, 30, 80, 255]);
    let bytes = html(content).select("div", 1).screenshot().unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (71, 29));
    assert_eq!(image.get_pixel(20, 20).0, [20, 180, 60, 255]);
    let bytes = html("<div></div>")
        .window_size(19, 23)
        .hide_background()
        .screenshot()
        .unwrap();
    assert_eq!(
        image::load_from_memory(&bytes)
            .unwrap()
            .to_rgba8()
            .get_pixel(0, 0)
            .0[3],
        0
    );
    let bytes = html(content).clip(10, 15, 31, 17).screenshot().unwrap();
    assert_eq!(
        (
            image::load_from_memory(&bytes).unwrap().width(),
            image::load_from_memory(&bytes).unwrap().height()
        ),
        (31, 17)
    );
    let bytes = html(content)
        .set_option(
            "clip",
            json!({"x":10,"y":15,"width":31,"height":17,"scale":2}),
        )
        .set_option("encoding", "base64")
        .screenshot()
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    assert_eq!((image.width(), image.height()), (62, 34));
}

#[test]
fn captures_full_page_jpeg_webp_base64_and_image_manipulations() {
    let content = "<style>body{margin:0;height:913px;background:#c02070}</style>";
    let bytes = html(content).window_size(213, 127).screenshot().unwrap();
    assert_eq!(image::load_from_memory(&bytes).unwrap().height(), 127);
    let bytes = html(content)
        .window_size(213, 127)
        .full_page()
        .screenshot()
        .unwrap();
    assert_eq!(image::load_from_memory(&bytes).unwrap().height(), 913);
    for format in ["jpeg", "webp"] {
        let bytes = html(content)
            .window_size(41, 37)
            .set_screenshot_type(format, Some(80))
            .screenshot()
            .unwrap();
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 41);
    }
    let bytes = html(content)
        .window_size(41, 37)
        .greyscale()
        .screenshot()
        .unwrap();
    let pixel = image::load_from_memory(&bytes)
        .unwrap()
        .to_rgb8()
        .get_pixel(0, 0)
        .0;
    assert_eq!(pixel[0], pixel[1]);
    assert_eq!(pixel[1], pixel[2]);
    let width = 17;
    let bytes = html(content)
        .manipulate(move |image| image.crop_imm(0, 0, width, 11))
        .manipulate(|image| image.rotate90())
        .screenshot()
        .unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    assert_eq!((image.width(), image.height()), (11, 17));
    assert!(
        html(content)
            .base64_screenshot()
            .unwrap()
            .starts_with("iVBOR")
    );
}

#[test]
fn prints_pdf_with_dimensions_headers_and_page_ranges() {
    let bytes = html("<h1>First</h1><div style='break-before:page'>Second</div>")
        .paper_size_with_unit(5.0, 7.0, Unit::In)
        .margins(5.0, 6.0, 7.0, 8.0)
        .show_browser_header_and_footer()
        .header_html("<span class='pageNumber'></span>")
        .hide_footer()
        .show_background()
        .tagged_pdf()
        .pages("2")
        .pdf()
        .unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    let pdf = String::from_utf8_lossy(&bytes);
    assert!(pdf.contains("/MediaBox [0 0 360 504]"), "{pdf}");
    assert!(pdf.contains("/Marked true"));
    assert!(html("hello").base64_pdf().unwrap().starts_with("JVBER"));
    let bytes = html("hello").format("A4").landscape(true).pdf().unwrap();
    let text = String::from_utf8_lossy(&bytes);
    let dimensions = text
        .split("/MediaBox [")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert!((dimensions[2] - 842.0).abs() < 1.0, "{dimensions:?}");
    assert!((dimensions[3] - 595.0).abs() < 1.0, "{dimensions:?}");
}

#[test]
fn performs_form_interactions_and_waits_for_dynamic_content() {
    let content = "<input id='name'><select id='state'><option value='BE'>Belgium</option><option value='MT'>Montana</option></select><button onclick=\"setTimeout(()=>{document.body.dataset.result=document.querySelector('input').value+':'+document.querySelector('select').value},50)\">Submit</button>";
    let value = html(content)
        .r#type("#name", "Zoë")
        .select_option("#state", "MT")
        .click("button")
        .wait_for_function(
            "document.body.dataset.result",
            Polling::Interval(Duration::from_millis(10)),
            Duration::from_secs(2),
        )
        .evaluate("document.body.dataset.result")
        .unwrap();
    assert_eq!(value, "Zoë:MT");
    let value = html(
        "<script>setTimeout(()=>{document.body.innerHTML='<p id=ready>Ready</p>'},50)</script>",
    )
    .wait_for_selector_with_options(
        "#ready",
        WaitForSelectorOptions {
            visible: true,
            ..Default::default()
        },
    )
    .evaluate("document.querySelector('#ready').textContent")
    .unwrap();
    assert_eq!(value, "Ready");
    let server = Server::start();
    let content = "<form action='/inspect' method='post'><input name='name'><select name='state'><option value='BE'>Belgium</option><option value='MT'>Montana</option></select><button>Submit</button></form>";
    let output = html(content)
        .set_content_url(&server.url)
        .r#type("input", "Alice")
        .select_option("select", "MT")
        .click("button")
        .body_html()
        .unwrap();
    assert!(output.contains("name=Alice&amp;state=MT"), "{output}");
}

#[test]
fn configures_scripts_media_devices_and_javascript() {
    assert_eq!(
        html("<script>window.observed=window.before</script>")
            .evaluate_on_new_document("window.before=42")
            .evaluate("window.observed")
            .unwrap(),
        42
    );
    assert_eq!(
        html("<script>window.ran=true</script>")
            .disable_javascript()
            .evaluate("typeof window.ran")
            .unwrap(),
        "undefined"
    );
    assert_eq!(
        html("hello")
            .emulate_media(Some("print"))
            .emulate_media_features(json!([{"name":"prefers-color-scheme","value":"dark"}]))
            .evaluate(
                "[matchMedia('print').matches,matchMedia('(prefers-color-scheme: dark)').matches]"
            )
            .unwrap(),
        json!([true, true])
    );
    assert_eq!(
        html("<meta name=viewport content='width=device-width'>")
            .device("iPhone X")
            .evaluate("[innerWidth,devicePixelRatio,'ontouchstart' in window]")
            .unwrap(),
        json!([375, 3, true])
    );
    assert_eq!(
        html("hello")
            .set_option("addScriptTag", json!({"content":"window.added=7"}))
            .set_option(
                "addStyleTag",
                json!({"content":"body{color:rgb(10,20,30)}"})
            )
            .evaluate("[window.added,getComputedStyle(document.body).color]")
            .unwrap(),
        json!([7, "rgb(10, 20, 30)"])
    );
    assert_eq!(
        html("<script>alert('stop')</script><p>Ready</p>")
            .dismiss_dialogs()
            .evaluate("document.querySelector('p').textContent")
            .unwrap(),
        "Ready"
    );
}

#[test]
fn sends_post_cookies_and_headers_and_authenticates() {
    let server = Server::start();
    let value = server
        .shot("/inspect")
        .post([("z", "last"), ("a", "first")])
        .use_cookies([("session", "abc")], None)
        .set_extra_http_headers([("X-Every", "every")])
        .set_extra_navigation_http_headers([("X-Navigation", "nav")])
        .evaluate("JSON.parse(document.querySelector('pre').textContent)")
        .unwrap();
    assert_eq!(value["method"], "POST");
    assert_eq!(value["body"], "z=last&a=first");
    assert_eq!(value["headers"]["cookie"], "session=abc");
    assert_eq!(value["headers"]["x-every"], "every");
    assert!(value["headers"]["x-navigation"].is_null());
    let headers = server
        .shot("/inspect")
        .set_extra_navigation_http_headers([("X-Navigation", "nav")])
        .evaluate("JSON.parse(document.querySelector('pre').textContent).headers")
        .unwrap();
    assert_eq!(headers["x-navigation"], "nav");
    assert_eq!(
        server
            .shot("/auth")
            .authenticate("user", "pass")
            .evaluate("document.body.textContent")
            .unwrap(),
        "authenticated"
    );
}

#[test]
fn reuses_a_persistent_profile_without_losing_browser_storage() {
    let server = Server::start();
    let profile = tempfile::tempdir().unwrap();
    let path = profile.path().to_str().unwrap();
    server
        .shot("/")
        .add_chromium_arguments([format!("user-data-dir={path}")])
        .evaluate("localStorage.setItem('saved', 'profile-value')")
        .unwrap();
    assert_eq!(
        server
            .shot("/")
            .user_data_dir(path)
            .evaluate("localStorage.getItem('saved')")
            .unwrap(),
        "profile-value"
    );
}

#[test]
fn captures_redirects_errors_and_blocks_requests() {
    let server = Server::start();
    let mut shot = server.shot("/redirect");
    assert!(shot.body_html().unwrap().contains("Final"));
    let history = shot.redirect_history().unwrap();
    assert_eq!(
        history
            .iter()
            .map(|entry| entry["status"].as_f64().unwrap() as u16)
            .collect::<Vec<_>>(),
        vec![302, 200]
    );
    let mut shot = server
        .shot("/")
        .block_urls(["asset.js"])
        .disable_images()
        .wait_until_network_idle(true)
        .set_delay(30);
    assert_eq!(
        shot.evaluate("[typeof window.assetLoaded,document.images.length]")
            .unwrap(),
        json!(["undefined", 0])
    );
    assert!(shot.failed_requests().unwrap().iter().any(|entry| {
        entry["url"]
            .as_str()
            .is_some_and(|url| url.ends_with("/asset.js"))
    }));
    let mut shot = server.shot("/missing").prevent_unsuccessful_response(true);
    assert!(matches!(
        shot.body_html(),
        Err(Error::UnsuccessfulResponse(404))
    ));
    assert_eq!(shot.failed_requests().unwrap()[0]["status"], 404);
    assert!(
        server
            .shot("/redirect")
            .disable_redirects()
            .body_html()
            .is_err()
    );
}

#[test]
fn renders_html_at_a_content_url_and_blocks_local_files_by_default() {
    let server = Server::start();
    let value = html("<p>Replacement</p><script>window.originSeen=location.origin</script>")
        .set_content_url(&server.url)
        .evaluate("[document.querySelector('p').textContent,window.originSeen]")
        .unwrap();
    assert_eq!(value, json!(["Replacement", server.url]));
    let directory = tempfile::tempdir().unwrap();
    let asset = directory.path().join("asset.js");
    std::fs::write(&asset, "window.localAsset='secret'").unwrap();
    let content = format!(
        "<script src='{}'></script>",
        url::Url::from_file_path(&asset).unwrap()
    );
    assert_eq!(
        html(&content).evaluate("typeof window.localAsset").unwrap(),
        "undefined"
    );
    assert_eq!(
        html(&content)
            .allow_local_file_access(true)
            .evaluate("window.localAsset")
            .unwrap(),
        "secret"
    );
    let page = directory.path().join("index.html");
    std::fs::write(&page, "<p>Local</p>").unwrap();
    assert!(
        Browsershot::html_from_file_path(&page)
            .no_sandbox()
            .body_html()
            .unwrap()
            .contains("Local")
    );
    assert_eq!(
        Browsershot::html_from_file_path(&page)
            .no_sandbox()
            .set_content_url(&server.url)
            .evaluate("[document.querySelector('p').textContent,location.origin]")
            .unwrap(),
        json!(["Local", server.url])
    );
}

#[test]
fn reports_invalid_input_missing_elements_and_deadlines() {
    assert!(matches!(
        Browsershot::url("file:///etc/passwd").body_html(),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        html("hello").select(".missing", 0).screenshot(),
        Err(Error::ElementNotFound(_))
    ));
    assert!(matches!(
        html("hello").set_option("notAnOption", true).pdf(),
        Err(Error::InvalidInput(_))
    ));
    let stalled = html("hello").timeout(3).evaluate("new Promise(()=>{})");
    assert!(matches!(stalled, Err(Error::Timeout)), "{stalled:?}");
    assert!(matches!(
        html("hello")
            .set_remote_instance("127.0.0.1", 1)
            .throw_on_remote_connection_error(true)
            .pdf(),
        Err(Error::RemoteConnection(_))
    ));
}

#[test]
fn preserves_output_results_exceptions_and_php_option_names() {
    let mut shot = html("<script>console.log('retained')</script>")
        .set_option("disableCaptureURLS", true)
        .set_option("args", json!(["--user-agent=Raw flag agent"]))
        .set_option("env", json!({"BROWSERSHOT_TEST": "present"}))
        .set_option("waitUntil", json!(["load", "networkidle0"]))
        .set_option("networkIdleTimeout", 60);
    assert_eq!(
        shot.evaluate("[21 * 2,navigator.userAgent]").unwrap(),
        json!([42, "Raw flag agent"])
    );
    let output = serde_json::to_value(shot.output().unwrap()).unwrap();
    assert_eq!(output["result"], json!([42, "Raw flag agent"]));
    assert!(output["exception"].is_null());
    assert!(shot.triggered_requests().unwrap().is_empty());
    assert!(
        shot.evaluate("throw new Error('expected failure')")
            .is_err()
    );
    let output = serde_json::to_value(shot.output().unwrap()).unwrap();
    assert!(output["result"].is_null());
    assert!(
        output["exception"]
            .as_str()
            .unwrap()
            .contains("expected failure")
    );
    assert_eq!(shot.console_messages().unwrap()[0]["message"], "retained");
}

#[test]
fn combines_named_devices_with_explicit_viewport_overrides() {
    let value = html("<meta name='viewport' content='width=device-width'>")
        .user_agent("Overridden by the device")
        .device("iPhone X")
        .window_size(431, 719)
        .device_scale_factor(2)
        .set_option("viewport.hasTouch", false)
        .evaluate("[innerWidth,innerHeight,devicePixelRatio,navigator.maxTouchPoints,navigator.userAgent.includes('iPhone')]")
        .unwrap();
    assert_eq!(value, json!([431, 719, 2, 0, true]));
}

#[test]
fn typing_dispatches_keyboard_events_instead_of_only_inserting_text() {
    let value = html("<input id='name'><script>window.keys=[];document.querySelector('input').addEventListener('keydown',e=>keys.push(e.key))</script>")
        .r#type("#name", "Az!")
        .evaluate("[document.querySelector('input').value,window.keys]")
        .unwrap();
    assert_eq!(value, json!(["Az!", ["A", "z", "!"]]));
}

#[test]
fn validates_html_like_php_without_blocking_explicit_http_resources() {
    for content in [
        "<iframe src='view-source:https://example.com'>",
        "<iframe src='&#118;iew-source:https://example.com'>",
        "<iframe src='//localhost/file'>",
        "<iframe src='//127.1.2.3/file'>",
        "<iframe src='&#47;&#47;[::1]/file'>",
        r"<iframe src='\\localhost/file'>",
        r"<iframe src='\\0.0.0.0/file'>",
        r"<iframe src='\\localhost\file'>",
        r"<iframe src='\\[::1]\file'>",
    ] {
        assert!(
            matches!(
                html(content).allow_local_file_access(true).body_html(),
                Err(Error::InvalidInput(_))
            ),
            "{content}"
        );
    }
    let server = Server::start();
    let content = format!(
        "<script src='{}/asset.js'></script><style>.hover\\:flex{{display:flex}}</style>",
        server.url
    );
    assert_eq!(html(&content).evaluate("window.assetLoaded").unwrap(), true);
}

#[test]
fn reports_console_values_page_exceptions_and_output_write_failures() {
    let mut shot = html("<script>console.warn(null,undefined,{a:1},[2]);throw null;</script>");
    shot.body_html().unwrap();
    let messages = shot.console_messages().unwrap();
    assert_eq!(messages[0]["type"], "warn");
    assert_eq!(
        messages[0]["message"],
        "null undefined JSHandle@object JSHandle@array"
    );
    assert!(messages[0]["stackTrace"].as_array().unwrap()[0]["url"].is_string());
    assert_eq!(shot.page_errors().unwrap()[0]["message"], "null");
    let directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        shot.save(directory.path().join("missing/output.png")),
        Err(Error::Io(_))
    ));
    assert!(shot.output().unwrap().exception.is_some());
}

#[test]
fn uses_authenticated_proxies_and_generates_pdf_outlines() {
    let proxy = Server::start();
    assert_eq!(
        Browsershot::url("http://proxy-fixture.invalid/")
            .no_sandbox()
            .timeout(15)
            .set_proxy_server(&proxy.url)
            .authenticate("user", "pass")
            .evaluate("document.querySelector('h1').textContent")
            .unwrap(),
        "Authenticated proxy"
    );
    let bytes = html("<h1>Outline heading</h1><p>Content</p>")
        .set_option("outline", true)
        .set_option("waitForFonts", false)
        .pdf()
        .unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("/Outlines"));
    let bytes = html("<style>body{background:#c03420}</style>")
        .set_screenshot_type("jpeg", Some(93))
        .manipulate(|image| image.to_rgba8().into())
        .screenshot()
        .unwrap();
    assert!(bytes.starts_with(&[0xff, 0xd8]));
}

#[test]
fn waits_for_async_functions_visible_boxes_and_enabled_locators() {
    let result = html("<div id='empty' style='width:0;height:0'></div><button disabled onclick='window.clicked=true'>Click</button><script>setTimeout(()=>document.querySelector('button').disabled=false,200)</script>")
        .locator_click("button")
        .wait_for_function("async () => { await new Promise(r=>setTimeout(r,50)); return window.clicked===true; }",Polling::RequestAnimationFrame,Duration::from_secs(2))
        .wait_for_selector_with_options("#empty",WaitForSelectorOptions{hidden:true,timeout:Some(Duration::from_millis(200)),..Default::default()})
        .evaluate("window.clicked").unwrap();
    assert_eq!(result, true);
}

#[test]
fn renders_over_a_devtools_pipe() {
    assert_eq!(
        html("<h1>Pipe</h1>")
            .use_pipe()
            .evaluate("document.querySelector('h1').textContent")
            .unwrap(),
        "Pipe"
    );
}

#[test]
fn distinguishes_strict_network_idle_from_two_connection_idle() {
    let server = Server::start();
    let content = "<script>setInterval(()=>fetch('/inspect'),100)</script>";
    assert_eq!(
        html(content)
            .set_content_url(&server.url)
            .wait_until_network_idle(false)
            .evaluate("42")
            .unwrap(),
        42
    );
    assert!(matches!(
        html(content)
            .set_content_url(&server.url)
            .wait_until_network_idle(true)
            .timeout(2)
            .evaluate("42"),
        Err(Error::Timeout)
    ));
    assert_eq!(
        html(content)
            .set_content_url(&server.url)
            .wait_until_network_idle(true)
            .set_option("networkIdleTimeout", 30)
            .timeout(5)
            .evaluate("42")
            .unwrap(),
        42
    );
}

#[test]
fn dispatches_repeated_clicks_and_waits_for_locators() {
    let content = "<button onclick='window.count=(window.count||0)+1'>Count</button><script>setTimeout(()=>{const e=document.createElement('button');e.id='later';e.onclick=()=>window.laterCount=(window.laterCount||0)+1;document.body.appendChild(e)},50)</script>";
    let result = html(content)
        .click_with_options(
            "button",
            ClickOptions {
                click_count: 3,
                ..Default::default()
            },
        )
        .locator_click_with_options(
            "#later",
            ClickOptions {
                click_count: 2,
                ..Default::default()
            },
        )
        .evaluate("[window.count,window.laterCount]")
        .unwrap();
    assert_eq!(result, json!([3, 2]));
}

struct RemoteChrome {
    child: std::process::Child,
    _profile: tempfile::TempDir,
    port: u16,
    websocket: String,
}
impl Drop for RemoteChrome {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl RemoteChrome {
    fn start() -> Self {
        let profile = tempfile::tempdir().unwrap();
        let executable = std::env::var_os("CHROME_PATH").unwrap_or_else(|| {
            if cfg!(target_os = "macos") {
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into()
            } else if cfg!(windows) {
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".into()
            } else {
                "google-chrome".into()
            }
        });
        let child = std::process::Command::new(executable)
            .args([
                "--headless=new",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--remote-debugging-port=0",
            ])
            .arg(format!("--user-data-dir={}", profile.path().display()))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let mut browser = Self {
            child,
            _profile: profile,
            port: 0,
            websocket: String::new(),
        };
        let start = std::time::Instant::now();
        loop {
            assert!(
                start.elapsed() < Duration::from_secs(15),
                "remote Chrome did not start"
            );
            assert!(
                browser.child.try_wait().unwrap().is_none(),
                "remote Chrome exited"
            );
            if let Ok(contents) =
                std::fs::read_to_string(browser._profile.path().join("DevToolsActivePort"))
            {
                let mut lines = contents.lines();
                if let (Some(port), Some(path)) = (lines.next(), lines.next()) {
                    browser.port = port.parse().unwrap();
                    browser.websocket = format!("ws://127.0.0.1:{port}{path}");
                    return browser;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn targets(&self) -> std::collections::BTreeSet<String> {
        let targets: Vec<serde_json::Value> =
            ureq::get(format!("http://127.0.0.1:{}/json/list", self.port))
                .call()
                .unwrap()
                .body_mut()
                .read_json()
                .unwrap();
        targets
            .iter()
            .filter(|target| target["type"] == "page")
            .map(|target| target["id"].as_str().unwrap().to_owned())
            .collect()
    }
}

#[test]
fn connects_to_remote_http_and_websocket_without_closing_the_remote_browser() {
    let mut browser = RemoteChrome::start();
    let before = browser.targets();
    assert_eq!(
        html("<h1>Remote HTTP</h1>")
            .set_remote_instance("127.0.0.1", browser.port)
            .throw_on_remote_connection_error(true)
            .evaluate("document.querySelector('h1').textContent")
            .unwrap(),
        "Remote HTTP"
    );
    assert_eq!(
        html("<h1>Remote WebSocket</h1>")
            .set_ws_endpoint(&browser.websocket)
            .throw_on_remote_connection_error(true)
            .evaluate("document.querySelector('h1').textContent")
            .unwrap(),
        "Remote WebSocket"
    );
    assert!(browser.child.try_wait().unwrap().is_none());
    let start = std::time::Instant::now();
    while browser.targets() != before {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "remote targets were not cleaned up: before={before:?}, after={:?}",
            browser.targets()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        html("local fallback")
            .set_remote_instance("127.0.0.1", 1)
            .evaluate("document.body.textContent")
            .unwrap(),
        "local fallback"
    );
}
