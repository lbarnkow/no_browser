//! Module containing the main [`Browser`][Browser] struct.

use super::page::Page;
use crate::{
    form::{self, Form},
    page,
};
use reqwest::{
    blocking::{Client, Response},
    Certificate, Method,
};
use thiserror::Error;

/// An error occurred while building the browser or executing actions.
#[derive(Debug, Error)]
pub enum Error {
    /// The underlying [reqwest `Client`](https://crates.io/crates/reqwest) could not be built.
    #[error("Failed to construct the http client!")]
    ConstructHttpClientError {
        /// The underlying error.
        #[source]
        source: reqwest::Error,
    },

    /// There was an error with [reqwest](https://crates.io/crates/reqwest) while sending the web request or due to
    /// exhausting the maximum number of redirects.
    #[error("Failed to send the request or redirection limit hit!")]
    SendRequestError {
        /// The underlying error.
        #[source]
        source: reqwest::Error,
    },

    /// The server response could not be decoded by [reqwest](https://crates.io/crates/reqwest).
    #[error("Failed to decode repsonse body!")]
    ResponseBodyDecodeError {
        /// The underlying error.
        #[source]
        source: reqwest::Error,
    },

    /// There was an error while building the [`Page`][Page] from the decoded http response.
    #[error("{source}")]
    PageError {
        /// The underlying error.
        #[from]
        source: page::Error,
    },

    /// There was an error in [`Form`][Form] while compiling the information needed to submit the form.
    #[error("{source}")]
    FormError {
        /// The underlying error.
        #[from]
        source: form::Error,
    },
}

/// Short-hand for `std::result::Result<T, no_browser::browser::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// A `light-weight` browser wrapped around a [reqwest `Client`](https://crates.io/crates/reqwest) to navigate to web
/// pages and submit forms.
///
/// Use `Browser::builder()` to initialize an instance.
///
/// # Example
///
/// ```
/// use no_browser::Browser;
///
/// let browser = Browser::builder().finish()?;
///
/// // Lets go to the Wikipedia main page
/// let page = browser.navigate_to("https://en.wikipedia.org/", None)?;
///
/// # Ok::<(), no_browser::browser::Error>(())
/// ```
#[derive(Debug)]
pub struct Browser {
    client: Client,
}

impl Browser {
    /// Return a [`BrowserBuilder`][BrowserBuilder] to initialize a [`Browser`][Browser] instance.
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder::new()
    }

    /// Navigate to a given `url`, optionally appending `query` parameters. Upon success the http response is decoded
    /// and used to initialize and return a [`Page`][Page] instance.
    pub fn navigate_to(&self, url: &str, query: Option<&Vec<(&str, &str)>>) -> Result<Page> {
        let mut rb = self.client.get(url);

        if let Some(query_value) = query {
            rb = rb.query(query_value)
        }

        let resp = rb
            .send()
            .map_err(|error| Error::SendRequestError { source: error })?;

        Self::build_page(Method::GET, resp)
    }

    /// Uses this [`Browser`][Browser] instance to submit a given `form` using a specific input/button
    /// (`submit_button_name`). Upon success the http response is decoded and used to initialize and return a
    /// [`Page`][Page] instance.
    pub fn submit_form(&self, form: &Form, submit_button_name: Option<&str>) -> Result<Page> {
        let info = form.submit(submit_button_name)?;

        let rb = if info.method == Method::GET {
            self.client.get(info.url).query(&info.data)
        } else {
            self.client.post(info.url).form(&info.data)
        };

        let resp = rb
            .send()
            .map_err(|error| Error::SendRequestError { source: error })?;

        Self::build_page(info.method, resp)
    }

    fn build_page(method: Method, resp: Response) -> Result<Page> {
        let url = resp.url().clone();
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp
            .text()
            .map_err(|error| Error::ResponseBodyDecodeError { source: error })?;

        Ok(Page::build(method, url, status, headers, text))
    }
}

/// A builder to initialize a [`Browser`][Browser] instance. It allows tweaking advanced settings for the http client.
/// Refer to the documentation of the public methods to learn about the available settings and their defaults. Use
/// `finish()` to get the configured [`Browser`][Browser].
#[derive(Debug)]
pub struct BrowserBuilder {
    cookie_store: bool,
    skip_tls_verify: bool,
    certs: Vec<Certificate>,
}

impl BrowserBuilder {
    fn new() -> Self {
        BrowserBuilder {
            cookie_store: true,
            skip_tls_verify: false,
            certs: Vec::new(),
        }
    }

    /// Set whether this [`Browser`][Browser] should have a cookie store and therefore handle cookies. Defaults to
    /// `true`.
    pub fn cookie_store(mut self, cookie_store: bool) -> Self {
        self.cookie_store = cookie_store;
        self
    }

    /// Set whether the verification of server certificates for TLS secured web sites should be skipped. Defaults to
    /// `false`, as this severely weakens security!
    ///
    /// Please look into function `add_cert()` before disabling verification!
    ///
    /// This setting may be useful in corporate environments with proxy servers controlling internet access. Often,
    /// these proxies act as a man-in-the-middle, decrypting the connection to the target web site to scan for viruses
    /// or malware, while re-encrypting the connection to your client with a custom self-signed certificate. In this
    /// particular scenario it may be useful while prototyping to disable verification and simply trust the connection.
    pub fn skip_tls_verify(mut self, skip_tls_verify: bool) -> Self {
        self.skip_tls_verify = skip_tls_verify;
        self
    }

    /// Adds an additional CA certificate into the trust store of the
    /// [reqwest `Client`](https://crates.io/crates/reqwest) to be used to verify server certificates when initiating
    /// TLS-secured connections. Use crates [rustls](https://crates.io/crates/rustls) and
    /// [rustls-pemfile](https://crates.io/crates/rustls-pemfile) to add certificates in DER or PEM format respectively.
    ///
    /// By default all [webpki-roots certificates](https://crates.io/crates/webpki-roots) are trusted by
    /// [rustls](https://crates.io/crates/rustls).
    pub fn add_cert(mut self, cert: Certificate) -> Self {
        self.certs.push(cert);
        self
    }

    /// Completes configuration of the [reqwest `Client`](https://crates.io/crates/reqwest) and returns the
    /// [`Browser`][Browser].
    pub fn finish(self) -> Result<Browser> {
        let mut client = reqwest::blocking::ClientBuilder::new().cookie_store(self.cookie_store);

        if self.skip_tls_verify {
            client = client.danger_accept_invalid_certs(true);
        }

        for cert in self.certs {
            client = client.add_root_certificate(cert);
        }

        let client = client
            .build()
            .map_err(|error| Error::ConstructHttpClientError { source: error })?;

        Ok(Browser { client })
    }
}

#[cfg(test)]
mod tests {
    use crate::{browser::Browser, input::InputType};
    use std::{collections::HashMap, net::SocketAddr, thread};
    use tiny_http::{Response, Server};

    static WEB_PAGE: &str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>tiny_http</title>
</head>
<body>
    <h1>Method</h1>
    <p id="method">{REQUEST_METHOD}</p>
    <h1>URL</h1>
    <p id="url">{REQUEST_URL}</p>
    <h1>Path</h1>
    <p id="path">{REQUEST_PATH}</p>
    <h1>Headers</h1>
    <ul>{REQUEST_HEADERS}
    </ul>
    <h1>Query parameters</h1>
    <ul>{REQUEST_QUERY}
    </ul>
    <h1>Payload</h1>
    <p id="payload">{REQUEST_PAYLOAD}</p>
    <h1>Form</h1>{FORM}
</body>
</html>
"#;

    static HEADER_ITEM: &str = r#"
        <li class="header">{HEADER}</li>"#;

    static QUERY_ITEM: &str = r#"
        <li class="query">{KEY}={VALUE}</li>"#;

    static FORM: &str = r#"
    <form id="form" action="{FORM_ACTION}" method="{FORM_METHOD}">
        <input type="text" name="text" value="">
        <button type="submit" name="submit" value="submit">SUBMIT</button>
    </form>
    "#;

    fn echo_server(requests: u64) -> SocketAddr {
        let server = Server::http("0.0.0.0:0").unwrap();
        let addr = server.server_addr();

        thread::spawn(move || {
            for _ in 0..requests {
                let mut request = server.incoming_requests().next().unwrap();

                let mut payload = String::new();
                request.as_reader().read_to_string(&mut payload).unwrap();

                let method = request.method().as_str();
                let url = urlencoding::decode(request.url()).unwrap().into_owned();
                let path = path(&url);
                let mut query = query(&url);

                let mut header_list = String::new();
                request.headers().iter().for_each(|header| {
                    let item = HEADER_ITEM.replace("{HEADER}", &header.to_string());
                    header_list.push_str(&item);
                });

                let mut query_list = String::new();
                for (k, v) in query.iter() {
                    let item = QUERY_ITEM.replace("{KEY}", k).replace("{VALUE}", v);
                    query_list.push_str(&item);
                }

                let form_action = query.remove("action").or(Some("form".to_owned())).unwrap();
                let form_method = query.remove("method").or(Some("get".to_owned())).unwrap();
                let form = FORM
                    .replace("{FORM_ACTION}", &form_action)
                    .replace("{FORM_METHOD}", &form_method);

                let html = WEB_PAGE
                    .replace("{REQUEST_METHOD}", method)
                    .replace("{REQUEST_URL}", &url)
                    .replace("{REQUEST_PATH}", &path)
                    .replace("{REQUEST_HEADERS}", &header_list)
                    .replace("{REQUEST_QUERY}", &query_list)
                    .replace("{REQUEST_PAYLOAD}", &payload)
                    .replace("{FORM}", &form);

                let mut response = Response::from_string(html);

                let header = tiny_http::Header::from_bytes(
                    &b"Set-Cookie"[..],
                    &b"NO_PATH_COOKIE=present; HttpOnly; SameSite=Strict"[..],
                )
                .unwrap();
                response.add_header(header);

                let header = tiny_http::Header::from_bytes(
                    &b"Set-Cookie"[..],
                    &b"ROOT_PATH_COOKIE=present; HttpOnly; Path=/; SameSite=Strict"[..],
                )
                .unwrap();
                response.add_header(header);

                if url.contains("/test") {
                    let header = tiny_http::Header::from_bytes(
                        &b"Set-Cookie"[..],
                        &b"TEST_PATH_COOKIE=present; HttpOnly; Path=/test; SameSite=Strict"[..],
                    )
                    .unwrap();
                    response.add_header(header);
                }

                request.respond(response).expect("Shouldn't fail here.");
            }
        });

        addr.to_ip().unwrap()
    }

    fn path(url: &str) -> String {
        let params = url.find('?');
        let fragment = url.rfind('#');
        let end = params.or(fragment);
        substring(url, 0, end)
    }

    fn query(url: &str) -> HashMap<String, String> {
        let mut q = HashMap::new();

        let params = url.find('?');
        let fragment = url.rfind('#');

        if params.is_some() {
            let url = substring(url, params.unwrap() + 1, fragment);

            for param in url.split('&') {
                let mut param = param.split('=');
                q.insert(
                    param.next().unwrap().to_owned(),
                    param.next().unwrap().to_owned(),
                );
            }
        }

        q
    }

    fn substring(s: &str, start: usize, end: Option<usize>) -> String {
        let mut len = usize::MAX;
        if let Some(end) = end {
            if end < start {
                panic!(
                    "Substring end index ({end}) mustn't be less than the start index ({start})!"
                );
            }
            len = end - start;
        }
        s.chars().skip(start).take(len).collect()
    }

    fn count_occurences(haystack: &str, needle: &str) -> usize {
        let mut result = 0;

        let mut haystack = haystack.to_owned();
        let mut find = haystack.find(needle);
        while find.is_some() {
            result += 1;
            haystack = substring(haystack.as_str(), find.unwrap() + needle.len(), None);
            find = haystack.find(needle);
        }

        result
    }

    #[test]
    fn cookies_restricted_by_path_and_host() {
        let addr = echo_server(6);
        let b = Browser::builder().finish().unwrap();

        // for localhost
        // load /, assert no cookies sent
        let url = format!("http://localhost:{}/", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 0);

        // load /, assert two cookies sent: no_path + root_path
        let url = format!("http://localhost:{}/", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 2);
        assert!(response.contains("NO_PATH_COOKIE=present"));
        assert!(response.contains("ROOT_PATH_COOKIE=present"));

        // load /test, assert two cookies sent: no_path + root_path
        let url = format!("http://localhost:{}/test", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 2);
        assert!(response.contains("NO_PATH_COOKIE=present"));
        assert!(response.contains("ROOT_PATH_COOKIE=present"));

        // load /test, assert three cookies sent: no_path + root_path + test_path
        let url = format!("http://localhost:{}/test", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 3);
        assert!(response.contains("NO_PATH_COOKIE=present"));
        assert!(response.contains("ROOT_PATH_COOKIE=present"));
        assert!(response.contains("TEST_PATH_COOKIE=present"));

        // load /foo, assert two cookies sent: no_path + test_path
        let url = format!("http://localhost:{}/foo", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 2);
        assert!(response.contains("NO_PATH_COOKIE=present"));
        assert!(response.contains("ROOT_PATH_COOKIE=present"));

        // for 127.0.0.1
        // load /, assert no cookies sent!
        let url = format!("http://127.0.0.1:{}/", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("bla", "blub"), ("foo", "bar")]))
            .unwrap();
        let response = p.text();
        let cookies_sent = count_occurences(response, "_COOKIE");
        assert_eq!(cookies_sent, 0);
    }

    #[test]
    fn submit_form_via_get() {
        let addr = echo_server(6);
        let b = Browser::builder().finish().unwrap();

        let url = format!("http://localhost:{}/", addr.port());
        let p = b
            .navigate_to(
                &url,
                Some(&vec![
                    ("action", "/relative/form/submiss.ion"),
                    ("method", "get"),
                ]),
            )
            .unwrap();

        let form = p.form(0).unwrap();
        let text = form.input(InputType::Text, "text").unwrap();
        text.borrow_mut().set_value(Some("Testing".to_owned()));

        let p = b.submit_form(form, Some("submit")).unwrap();

        let method = p.select_first("p#method").unwrap();
        assert_eq!(method.inner_html(), "GET");

        let path = p.select_first("p#path").unwrap();
        assert_eq!(path.inner_html(), "/relative/form/submiss.ion");

        let submitted: Vec<String> = p
            .select("ul > li.query")
            .unwrap()
            .iter()
            .map(|e| e.inner_html())
            .collect();

        assert!(submitted.contains(&"text=Testing".to_owned()));
        assert!(submitted.contains(&"submit=submit".to_owned()));
    }

    #[test]
    fn submit_form_via_post() {
        let addr = echo_server(6);
        let b = Browser::builder().finish().unwrap();

        let url = format!("http://localhost:{}/", addr.port());
        let action = format!("http://127.0.0.1:{}/absolute/form/submiss.ion", addr.port());
        let p = b
            .navigate_to(&url, Some(&vec![("action", &action), ("method", "post")]))
            .unwrap();

        let form = p.form(0).unwrap();
        let text = form.input(InputType::Text, "text").unwrap();
        text.borrow_mut().set_value(Some("Testing".to_owned()));

        let p = b.submit_form(form, Some("submit")).unwrap();

        let method = p.select_first("p#method").unwrap();
        assert_eq!(method.inner_html(), "POST");

        let path = p.select_first("p#path").unwrap();
        assert_eq!(path.inner_html(), "/absolute/form/submiss.ion");

        let host: Vec<String> = p
            .select("ul > li.header")
            .unwrap()
            .iter()
            .map(|li| li.inner_html())
            .filter(|header| header.starts_with("host: "))
            .collect();
        assert_eq!(host.len(), 1);
        assert!(host[0].starts_with("host: 127.0.0.1:"));

        let submitted = p.select_first("p#payload").unwrap().inner_html();
        assert!(submitted.contains(&"text=Testing".to_owned()));
        assert!(submitted.contains(&"submit=submit".to_owned()));
    }
}
