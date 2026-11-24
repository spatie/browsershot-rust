#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod browser;
mod cdp;
mod options;
#[cfg(windows)]
mod windows_pipe;

pub use options::{ClickOptions, Polling, Unit, WaitForSelectorOptions};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, sync::Arc, time::Duration};

/// An error encountered while launching Chrome, navigating, or producing output.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An operating-system operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Chrome's WebSocket connection failed.
    #[error(transparent)]
    WebSocket(#[from] Box<tungstenite::Error>),
    /// A DevTools message was not valid JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Chrome rejected a DevTools command.
    #[error("DevTools: {0}")]
    Protocol(String),
    /// A browser operation exceeded its deadline.
    #[error("browser operation timed out")]
    Timeout,
    /// A URL, selector, or option was invalid.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// A requested element was not found or had no visible bounding box.
    #[error("element not found: {0}")]
    ElementNotFound(String),
    /// The main document returned an unsuccessful HTTP response.
    #[error("unsuccessful response: HTTP {0}")]
    UnsuccessfulResponse(u16),
    /// No Chrome executable could be found.
    #[error("Chrome not found; use set_chrome_path() or set CHROME_PATH")]
    ChromeNotFound,
    /// Chrome exited before its DevTools endpoint became available.
    #[error("Chrome exited during startup: {0}")]
    ChromeExited(String),
    /// A remote DevTools endpoint could not be reached.
    #[error("remote connection: {0}")]
    RemoteConnection(String),
    /// Chrome returned malformed binary output.
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    /// Image decoding or manipulation failed.
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

impl From<tungstenite::Error> for Error {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(error))
    }
}

/// The result of a browser operation, including requests and page diagnostics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChromiumResult {
    /// All requested URLs, in request order.
    pub requests: Vec<Value>,
    /// Console messages with their type, text, location and stack trace.
    pub console_messages: Vec<Value>,
    /// Unsuccessful HTTP responses and network failures.
    pub failed_requests: Vec<Value>,
    /// Main-frame responses, including redirects and the final response.
    pub redirect_history: Vec<Value>,
    /// Uncaught JavaScript exceptions.
    pub page_errors: Vec<Value>,
}

#[derive(Debug, Clone)]
enum Source {
    Url(String),
    Html(String),
    File(std::path::PathBuf),
}

/// Configure a headless Chrome page and render images, PDFs or HTML.
///
/// Builders consume and return themselves. Output methods borrow mutably so
/// diagnostics can be retrieved from the same operation with [`Self::output`].
/// Chrome must be installed separately; no Node runtime is used.
#[derive(Clone)]
#[must_use]
pub struct Browsershot {
    source: Source,
    options: Value,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
    temp_path: Option<std::path::PathBuf>,
    result: Option<ChromiumResult>,
    manipulations: Vec<Arc<dyn Fn(image::DynamicImage) -> image::DynamicImage + Send + Sync>>,
}

impl std::fmt::Debug for Browsershot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Browsershot")
            .field("source", &self.source)
            .field("options", &self.options)
            .field("arguments", &self.arguments)
            .field("temp_path", &self.temp_path)
            .field("result", &self.result)
            .field("manipulations", &self.manipulations.len())
            .finish_non_exhaustive()
    }
}

impl Browsershot {
    fn new(source: Source) -> Self {
        Self {
            source,
            options: json!({"viewport":{"width":800,"height":600,"deviceScaleFactor":1,"isMobile":false},"timeout":60000,"protocolTimeout":30000}),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            temp_path: None,
            result: None,
            manipulations: Vec::new(),
        }
    }

    /// Load a URL. Only HTTP and HTTPS URLs are accepted.
    pub fn url(url: impl Into<String>) -> Self {
        Self::new(Source::Url(url.into()))
    }

    /// Render HTML. Local file requests are blocked unless explicitly enabled.
    pub fn html(html: impl Into<String>) -> Self {
        Self::new(Source::Html(html.into()))
    }

    /// Load HTML from a file. File existence is checked when rendering.
    pub fn html_from_file_path(path: impl AsRef<Path>) -> Self {
        Self::new(Source::File(path.as_ref().into()))
    }

    /// Replace the current page with a URL.
    pub fn set_url(mut self, url: impl Into<String>) -> Self {
        self.source = Source::Url(url.into());
        self
    }

    /// Replace the current page with HTML.
    pub fn set_html(mut self, html: impl Into<String>) -> Self {
        self.source = Source::Html(html.into());
        self
    }

    /// Replace the current page with an HTML file.
    pub fn set_html_from_file_path(mut self, path: impl AsRef<Path>) -> Self {
        self.source = Source::File(path.as_ref().into());
        self
    }

    /// Set an option using PHP's option names, including dotted viewport keys.
    /// Unknown options are rejected when rendering rather than silently ignored.
    pub fn set_option(mut self, key: &str, value: impl Into<Value>) -> Self {
        let mut slot = &mut self.options;
        for part in key.split('.') {
            if !slot.is_object() {
                *slot = json!({});
            }
            slot = &mut slot[part];
        }
        *slot = value.into();
        self
    }

    /// Add Chromium flags. A missing `--` prefix is added automatically.
    pub fn add_chromium_arguments(
        mut self,
        arguments: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.arguments.extend(
            arguments
                .into_iter()
                .map(|arg| format!("--{}", arg.as_ref().trim_start_matches('-'))),
        );
        self
    }

    /// Set environment variables for the Chrome child process.
    pub fn set_environment_options(
        mut self,
        environment: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.environment.extend(
            environment
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Place temporary Chrome profiles and HTML files in this directory.
    pub fn set_custom_temp_path(mut self, path: impl AsRef<Path>) -> Self {
        self.temp_path = Some(path.as_ref().into());
        self
    }

    /// Apply a Rust image transformation after capturing the screenshot.
    /// Transformations run in insertion order for both file and in-memory output.
    pub fn manipulate(
        mut self,
        operation: impl Fn(image::DynamicImage) -> image::DynamicImage + Send + Sync + 'static,
    ) -> Self {
        self.manipulations.push(Arc::new(operation));
        self
    }

    /// Convert the screenshot to greyscale.
    pub fn greyscale(self) -> Self {
        self.manipulate(|image| image.grayscale())
    }

    /// Save a PDF for a `.pdf` extension, or a screenshot otherwise.
    ///
    /// # Errors
    /// Returns an error for invalid input, browser failures or file-writing errors.
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| Error::InvalidInput("output file must have an extension".into()))?;
        if extension.eq_ignore_ascii_case("pdf") {
            self.save_pdf(path)
        } else {
            std::fs::write(path, self.screenshot()?)?;
            Ok(())
        }
    }

    /// Save the page as a PDF, regardless of the file extension.
    ///
    /// # Errors
    /// Returns browser, PDF option validation or file-writing errors.
    pub fn save_pdf(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        std::fs::write(path, self.pdf()?)?;
        Ok(())
    }

    /// Capture the page as PNG, JPEG or WebP bytes.
    ///
    /// # Errors
    /// Returns browser, image manipulation or invalid screenshot option errors.
    pub fn screenshot(&mut self) -> Result<Vec<u8>, Error> {
        let data = self.run(browser::Action::Screenshot)?;
        let bytes = STANDARD.decode(
            data.as_str()
                .ok_or_else(|| Error::Protocol("missing screenshot data".into()))?,
        )?;
        if self.manipulations.is_empty() {
            return Ok(bytes);
        }
        let mut image = image::load_from_memory(&bytes)?;
        for operation in &self.manipulations {
            image = operation(image);
        }
        let format = match self.options["type"].as_str().unwrap_or("png") {
            "png" => image::ImageFormat::Png,
            "jpeg" => image::ImageFormat::Jpeg,
            "webp" => image::ImageFormat::WebP,
            other => {
                return Err(Error::InvalidInput(format!(
                    "unsupported screenshot type {other}"
                )));
            }
        };
        let mut output = std::io::Cursor::new(Vec::new());
        image.write_to(&mut output, format)?;
        Ok(output.into_inner())
    }

    /// Capture a base64-encoded screenshot.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::screenshot`].
    pub fn base64_screenshot(&mut self) -> Result<String, Error> {
        Ok(STANDARD.encode(self.screenshot()?))
    }

    /// Print the page to PDF bytes.
    ///
    /// # Errors
    /// Returns browser or invalid PDF option errors.
    pub fn pdf(&mut self) -> Result<Vec<u8>, Error> {
        Ok(STANDARD.decode(self.base64_pdf()?)?)
    }

    /// Print the page to a base64-encoded PDF.
    ///
    /// # Errors
    /// Returns browser or invalid PDF option errors.
    pub fn base64_pdf(&mut self) -> Result<String, Error> {
        self.run(browser::Action::Pdf)?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::Protocol("missing PDF data".into()))
    }

    /// Return the document HTML after JavaScript and configured interactions finish.
    ///
    /// # Errors
    /// Returns navigation, timeout or JavaScript evaluation errors.
    pub fn body_html(&mut self) -> Result<String, Error> {
        self.evaluate(r"(document.doctype ? new XMLSerializer().serializeToString(document.doctype) + '\n' : '') + document.documentElement.outerHTML")?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::Protocol("missing HTML".into()))
    }

    /// Evaluate a JavaScript expression or function, awaiting promises.
    ///
    /// # Errors
    /// Returns navigation, timeout, JavaScript exceptions or serialization errors.
    pub fn evaluate(&mut self, script: &str) -> Result<Value, Error> {
        self.run(browser::Action::Evaluate(script))
    }

    fn run(&mut self, action: browser::Action<'_>) -> Result<Value, Error> {
        self.result = Some(ChromiumResult::default());
        browser::run(self, action)
    }

    /// Diagnostics from the most recent operation, including a failed operation.
    pub fn output(&self) -> Option<&ChromiumResult> {
        self.result.as_ref()
    }

    fn diagnostics(&mut self) -> Result<&ChromiumResult, Error> {
        if self.result.is_none() {
            self.run(browser::Action::Diagnostics)?;
        }
        self.result
            .as_ref()
            .ok_or_else(|| Error::Protocol("missing diagnostics".into()))
    }

    /// Return requested URLs, reusing the last operation if available.
    ///
    /// # Errors
    /// Returns browser errors if no operation has run yet.
    pub fn triggered_requests(&mut self) -> Result<Vec<Value>, Error> {
        Ok(self.diagnostics()?.requests.clone())
    }

    /// Return console messages from the most recent operation.
    ///
    /// # Errors
    /// Returns browser errors if no operation has run yet.
    pub fn console_messages(&mut self) -> Result<Vec<Value>, Error> {
        Ok(self.diagnostics()?.console_messages.clone())
    }

    /// Return failed HTTP responses and failed network requests.
    ///
    /// # Errors
    /// Returns browser errors if no operation has run yet.
    pub fn failed_requests(&mut self) -> Result<Vec<Value>, Error> {
        Ok(self.diagnostics()?.failed_requests.clone())
    }

    /// Return main-frame redirect responses and the final response.
    ///
    /// # Errors
    /// Returns browser errors if no operation has run yet.
    pub fn redirect_history(&mut self) -> Result<Vec<Value>, Error> {
        Ok(self.diagnostics()?.redirect_history.clone())
    }

    /// Return uncaught JavaScript errors.
    ///
    /// # Errors
    /// Returns browser errors if no operation has run yet.
    pub fn page_errors(&mut self) -> Result<Vec<Value>, Error> {
        Ok(self.diagnostics()?.page_errors.clone())
    }

    fn duration(&self, key: &str) -> Duration {
        Duration::from_millis(self.options[key].as_u64().unwrap_or(60000))
    }
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
