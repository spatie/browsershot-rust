use crate::Browsershot;
use serde_json::{Value, json};
use std::time::Duration;

/// The unit used for PDF dimensions and margins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    /// Millimetres.
    #[default]
    Mm,
    /// Centimetres.
    Cm,
    /// Inches.
    In,
    /// CSS pixels (96 per inch).
    Px,
}

impl Unit {
    fn suffix(self) -> &'static str {
        match self {
            Self::Mm => "mm",
            Self::Cm => "cm",
            Self::In => "in",
            Self::Px => "px",
        }
    }
}

/// How frequently a JavaScript wait condition is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Polling {
    /// Check on animation frames.
    #[default]
    RequestAnimationFrame,
    /// Check when the DOM changes.
    Mutation,
    /// Check at a fixed interval.
    Interval(Duration),
}

/// Optional mouse click settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickOptions {
    /// Mouse button: `left`, `right`, or `middle`.
    pub button: String,
    /// Number of clicks.
    pub click_count: u32,
    /// Delay between mouse-down and mouse-up.
    pub delay: Duration,
}
impl Default for ClickOptions {
    fn default() -> Self {
        Self {
            button: "left".into(),
            click_count: 1,
            delay: Duration::ZERO,
        }
    }
}

/// Conditions for waiting on an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaitForSelectorOptions {
    /// Require the element to be visible.
    pub visible: bool,
    /// Wait until the element is hidden or removed instead.
    pub hidden: bool,
    /// Override the page timeout. `None` uses the page timeout.
    pub timeout: Option<Duration>,
}

macro_rules! flags {
    ($($name:ident, $key:literal, $doc:literal;)*) => { impl Browsershot { $(
        #[doc = $doc]
        pub fn $name(self) -> Self { self.set_option($key, true) }
    )* } };
}
flags! {
    full_page, "fullPage", "Capture the full scrollable page.";
    no_sandbox, "noSandbox", "Disable Chrome's sandbox. Only use in a trusted environment.";
    dismiss_dialogs, "dismissDialogs", "Automatically dismiss JavaScript dialogs.";
    disable_javascript, "disableJavascript", "Disable JavaScript execution in the page.";
    disable_images, "disableImages", "Block image requests and remove image elements.";
    disable_capture_urls, "disableCaptureURLs", "Stop collecting requested URLs to reduce memory usage.";
    disable_redirects, "disableRedirects", "Block HTTP redirects.";
    ignore_https_errors, "acceptInsecureCerts", "Accept invalid HTTPS certificates.";
    tagged_pdf, "tagged", "Generate an accessible tagged PDF.";
    transparent_background, "omitBackground", "Use a transparent PDF or screenshot background.";
    use_pipe, "pipe", "Use a local DevTools pipe instead of WebSocket.";
}

macro_rules! strings {
    ($($name:ident, $key:literal, $doc:literal;)*) => { impl Browsershot { $(
        #[doc = $doc]
        pub fn $name(self, value: impl Into<String>) -> Self { self.set_option($key, value.into()) }
    )* } };
}
strings! {
    set_chrome_path, "executablePath", "Set the Chrome or Chromium executable path.";
    set_proxy_server, "proxyServer", "Use this proxy server for Chrome requests.";
    user_agent, "userAgent", "Override the browser's user agent.";
    user_data_dir, "userDataDir", "Use a persistent absolute Chrome profile directory.";
    header_html, "headerTemplate", "Set the PDF header template.";
    footer_html, "footerTemplate", "Set the PDF footer template.";
    pages, "pageRanges", "Print only these page ranges, such as `1-3, 5`.";
    format, "format", "Set the paper format, such as `A4` or `Letter`.";
    set_ws_endpoint, "browserWSEndpoint", "Connect to a remote browser WebSocket endpoint.";
    set_content_url, "contentUrl", "Serve HTML at this URL, preserving its origin and relative assets.";
    device, "device", "Emulate a named Puppeteer device, such as `iPhone X`.";
    evaluate_on_new_document, "evaluateOnNewDocument", "Evaluate this function before scripts run on each new document.";
}

impl Browsershot {
    /// Set the viewport width and height in CSS pixels.
    pub fn window_size(self, width: u32, height: u32) -> Self {
        self.set_option("viewport.width", width)
            .set_option("viewport.height", height)
    }
    /// Set the device pixel ratio, clamped to Chrome's supported range 1–3.
    pub fn device_scale_factor(self, factor: u32) -> Self {
        self.set_option("viewport.deviceScaleFactor", factor.clamp(1, 3))
    }
    /// Emulate a mobile viewport.
    pub fn mobile(self) -> Self {
        self.set_option("viewport.isMobile", true)
    }
    /// Enable touch emulation.
    pub fn touch(self) -> Self {
        self.set_option("viewport.hasTouch", true)
    }
    /// Set landscape orientation.
    pub fn landscape(self, landscape: bool) -> Self {
        self.set_option("landscape", landscape)
    }
    /// Set the PDF scale (0.1 through 2).
    pub fn scale(self, scale: f64) -> Self {
        self.set_option("scale", scale)
    }
    /// Set the screenshot format and optional JPEG/WebP quality (0–100).
    pub fn set_screenshot_type(self, format: &str, quality: Option<u8>) -> Self {
        let builder = self.set_option("type", format);
        if let Some(quality) = quality {
            builder.set_option("quality", quality)
        } else {
            builder
        }
    }
    /// Set a screenshot crop in CSS pixels.
    pub fn clip(self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.set_option(
            "clip",
            json!({"x":x,"y":y,"width":width,"height":height,"scale":1}),
        )
    }
    /// Screenshot the indexed element matching a CSS selector.
    pub fn select(self, selector: &str, index: usize) -> Self {
        self.set_option("selector", selector).selector_index(index)
    }
    /// Change the index of the screenshot element.
    pub fn selector_index(self, index: usize) -> Self {
        self.set_option("selectorIndex", index)
    }
    /// Use millimetres for the paper width and height.
    pub fn paper_size(self, width: f64, height: f64) -> Self {
        self.paper_size_with_unit(width, height, Unit::Mm)
    }
    /// Set the paper width and height with explicit units.
    pub fn paper_size_with_unit(self, width: f64, height: f64, unit: Unit) -> Self {
        self.set_option("width", format!("{width}{}", unit.suffix()))
            .set_option("height", format!("{height}{}", unit.suffix()))
    }
    /// Set top, right, bottom and left PDF margins in millimetres.
    pub fn margins(self, top: f64, right: f64, bottom: f64, left: f64) -> Self {
        self.margins_with_unit(top, right, bottom, left, Unit::Mm)
    }
    /// Set PDF margins with explicit units.
    pub fn margins_with_unit(
        self,
        top: f64,
        right: f64,
        bottom: f64,
        left: f64,
        unit: Unit,
    ) -> Self {
        self.set_option("margin", json!({"top":format!("{top}{}",unit.suffix()),"right":format!("{right}{}",unit.suffix()),"bottom":format!("{bottom}{}",unit.suffix()),"left":format!("{left}{}",unit.suffix())}))
    }
    /// Include Chrome's PDF headers and footers.
    pub fn show_browser_header_and_footer(self) -> Self {
        self.set_option("displayHeaderFooter", true)
    }
    /// Hide Chrome's PDF headers and footers.
    pub fn hide_browser_header_and_footer(self) -> Self {
        self.set_option("displayHeaderFooter", false)
    }
    /// Hide only the PDF header.
    pub fn hide_header(self) -> Self {
        self.header_html("<p></p>")
    }
    /// Hide only the PDF footer.
    pub fn hide_footer(self) -> Self {
        self.footer_html("<p></p>")
    }
    /// Include background graphics in PDFs and screenshots.
    pub fn show_background(self) -> Self {
        self.set_option("printBackground", true)
            .set_option("omitBackground", false)
    }
    /// Omit PDF background graphics and use transparent screenshots.
    pub fn hide_background(self) -> Self {
        self.set_option("printBackground", false)
            .set_option("omitBackground", true)
    }
    /// Wait before producing output, in milliseconds.
    pub fn set_delay(self, milliseconds: u64) -> Self {
        self.set_option("delay", milliseconds)
    }
    /// Wait before producing output, in milliseconds.
    pub fn delay(self, milliseconds: u64) -> Self {
        self.set_delay(milliseconds)
    }
    /// Set the total operation timeout, in seconds.
    pub fn timeout(self, seconds: u64) -> Self {
        self.set_option("timeout", seconds.saturating_mul(1000))
    }
    /// Set the maximum duration of each DevTools command, in seconds.
    pub fn protocol_timeout(self, seconds: u64) -> Self {
        self.set_option("protocolTimeout", seconds.saturating_mul(1000))
    }
    /// Wait for 500 ms with zero (strict) or at most two open requests.
    pub fn wait_until_network_idle(self, strict: bool) -> Self {
        self.set_option(
            "waitUntil",
            if strict {
                "networkidle0"
            } else {
                "networkidle2"
            },
        )
    }
    /// Wait for a JavaScript expression to become truthy.
    pub fn wait_for_function(self, function: &str, polling: Polling, timeout: Duration) -> Self {
        let polling = match polling {
            Polling::RequestAnimationFrame => json!("raf"),
            Polling::Mutation => json!("mutation"),
            Polling::Interval(duration) => json!(duration.as_millis() as u64),
        };
        self.set_option("function", function)
            .set_option("functionPolling", polling)
            .set_option("functionTimeout", timeout.as_millis() as u64)
    }
    /// Wait for an element to exist.
    pub fn wait_for_selector(self, selector: &str) -> Self {
        self.wait_for_selector_with_options(selector, WaitForSelectorOptions::default())
    }
    /// Wait for an element with visibility and timeout conditions.
    pub fn wait_for_selector_with_options(
        self,
        selector: &str,
        options: WaitForSelectorOptions,
    ) -> Self {
        self.set_option("waitForSelector",selector).set_option("waitForSelectorOptions",json!({"visible":options.visible,"hidden":options.hidden,"timeout":options.timeout.map(|duration| duration.as_millis() as u64)}))
    }
    /// Set the CSS media type (`screen`, `print`, or `None` to reset).
    pub fn emulate_media(self, media: Option<&str>) -> Self {
        self.set_option("emulateMedia", json!(media))
    }
    /// Set CSS media features, such as `prefers-color-scheme`.
    pub fn emulate_media_features(self, features: Value) -> Self {
        self.set_option("emulateMediaFeatures", features)
    }
    /// Allow local file requests from trusted HTML.
    pub fn allow_local_file_access(self, allow: bool) -> Self {
        self.set_option("allowLocalFileAccess", allow)
    }
    /// Reject HTTP status codes 400–599 for the main document.
    pub fn prevent_unsuccessful_response(self, prevent: bool) -> Self {
        self.set_option("preventUnsuccessfulResponse", prevent)
    }
    /// Choose whether remote connection failures are fatal instead of launching locally.
    pub fn throw_on_remote_connection_error(self, throw: bool) -> Self {
        self.set_option("throwOnRemoteConnectionError", throw)
    }
    /// Connect to Chrome's HTTP debugging endpoint.
    pub fn set_remote_instance(self, host: &str, port: u16) -> Self {
        self.set_option("remoteInstanceUrl", format!("http://{host}:{port}"))
    }
    /// Supply HTTP authentication credentials, including proxy authentication challenges.
    pub fn authenticate(self, username: &str, password: &str) -> Self {
        self.set_option(
            "authentication",
            json!({"username":username,"password":password}),
        )
    }
    /// Send form fields using a POST navigation.
    pub fn post(
        self,
        params: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.set_option(
            "postParams",
            json!(
                params
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect::<std::collections::BTreeMap<String, String>>()
            ),
        )
    }
    /// Set cookies, using the URL host unless a domain is supplied.
    pub fn use_cookies(
        self,
        cookies: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        domain: Option<&str>,
    ) -> Self {
        let inferred = if let crate::Source::Url(url) = &self.source {
            url::Url::parse(url)
                .ok()
                .and_then(|url| url.host_str().map(str::to_owned))
        } else {
            None
        };
        let domain = domain.map(str::to_owned).or(inferred).unwrap_or_default();
        let mut values = self.options["cookies"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        values.extend(cookies.into_iter().map(|(name,value)|json!({"name":name.into(),"value":value.into(),"domain":domain,"path":"/"})));
        self.set_option("cookies", json!(values))
    }
    /// Add headers to every request.
    pub fn set_extra_http_headers(
        self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.set_option(
            "extraHTTPHeaders",
            json!(
                headers
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect::<std::collections::BTreeMap<String, String>>()
            ),
        )
    }
    /// Add headers only to document navigations, including frames.
    pub fn set_extra_navigation_http_headers(
        self,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.set_option(
            "extraNavigationHTTPHeaders",
            json!(
                headers
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect::<std::collections::BTreeMap<String, String>>()
            ),
        )
    }
    /// Block requests whose URLs contain any of these fragments.
    pub fn block_urls(self, urls: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.set_option(
            "blockUrls",
            json!(urls.into_iter().map(Into::into).collect::<Vec<String>>()),
        )
    }
    /// Block requests to these exact hostnames.
    pub fn block_domains(self, domains: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.set_option(
            "blockDomains",
            json!(domains.into_iter().map(Into::into).collect::<Vec<String>>()),
        )
    }
    /// Type into an input with no per-character delay.
    pub fn r#type(self, selector: &str, text: &str) -> Self {
        self.type_with_delay(selector, text, Duration::ZERO)
    }
    /// Type into an input with a per-character delay.
    pub fn type_with_delay(mut self, selector: &str, text: &str, delay: Duration) -> Self {
        self.push_option(
            "types",
            json!({"selector":selector,"text":text,"delay":delay.as_millis() as u64}),
        );
        self
    }
    /// Select a value in a dropdown.
    pub fn select_option(mut self, selector: &str, value: &str) -> Self {
        self.push_option("selects", json!({"selector":selector,"value":value}));
        self
    }
    /// Click a matching element using the left mouse button.
    pub fn click(self, selector: &str) -> Self {
        self.click_with_options(selector, ClickOptions::default())
    }
    /// Click a matching element with explicit mouse settings.
    pub fn click_with_options(mut self, selector: &str, options: ClickOptions) -> Self {
        self.push_option("clicks",json!({"selector":selector,"button":options.button,"clickCount":options.click_count,"delay":options.delay.as_millis() as u64}));
        self
    }
    /// Wait for an element to become visible before clicking it.
    pub fn locator_click(self, selector: &str) -> Self {
        self.locator_click_with_options(selector, ClickOptions::default())
    }
    /// Wait for a visible element and click it with explicit mouse settings.
    pub fn locator_click_with_options(mut self, selector: &str, options: ClickOptions) -> Self {
        self.push_option(
            "locatorClicks",
            json!({"selector":selector,"button":options.button,"clickCount":options.click_count,"delay":options.delay.as_millis() as u64}),
        );
        self
    }
    /// Start the visible PDF page numbering at this number.
    pub fn initial_page_number(self, number: u32) -> Self {
        self.set_option("initialPageNumber", number.saturating_sub(1))
            .pages(format!("{number}-"))
    }
    fn push_option(&mut self, key: &str, value: Value) {
        if !self.options[key].is_array() {
            self.options[key] = json!([]);
        }
        if let Some(values) = self.options[key].as_array_mut() {
            values.push(value);
        }
    }
}
