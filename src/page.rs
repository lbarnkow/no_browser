//! Module containing the [`Page`][Page] struct.

use crate::form::Form;
use reqwest::{header::HeaderMap, Method, StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

/// An error occurred while working with the page.
#[derive(Debug, Error)]
pub enum Error {
    /// The requested query param is not part of this page's query string.
    #[error("Query param '{param}' is not defined in query string '{query}'!")]
    UnknownQueryParamError {
        /// The actual query string used to fetch this page.
        query: String,
        /// The `param` missing from the query string.
        param: String,
    },

    /// The given [CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors) could not be parsed.
    #[error("Failed to parse CSS selector '{selector}', reason: {reason}")]
    CssSelectorParseError {
        /// The given `selector` that could not be parsed.
        selector: String,
        /// The `reason` given by the parser.
        reason: String,
    },

    /// The given CSS selector matched nothing on this page.
    #[error("CSS selector '{selector}' matched no elements.")]
    CssSelectorResultEmptyError {
        /// The given `selector` that had no matches.
        selector: String,
    },

    /// The given form index is out of bounds.
    #[error("This page contains {num_forms} forms; index {idx} is out of bounds!")]
    FormIndexOutOfBoundsError {
        /// The number of forms on this page.
        num_forms: usize,
        /// The out-of-bounds index.
        idx: usize,
    },

    /// No form found for the given `id`.
    #[error("This page contains no form with id '{id}'!")]
    FormIdNotFoundError {
        /// The `id` that matched no form.
        id: String,
    },
}

/// Short-hand for `std::result::Result<T, no_browser::page::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Struct [`Page`][Page] represents a loaded and parsed web response.
///
/// It gives access to:
/// * response meta data, like http method (`method()`) used to access the page url (`url()`), the http response status
///   (`status()`) and response headers (`headers()`);
/// * the unprocessed reponse body (`text()`);
/// * individual query parameters form the page's url (`query()`);
/// * parsed html elements via [CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors) either
///   by returning all matches (`select()`) or returning the first match only (`select_first()`);
/// * parsed html forms identified either by index (`form()`) or by id (`form_by_id()`);
///
/// See the main docs of [crate `no_browser`][crate] for usage examples.
#[derive(Debug)]
pub struct Page {
    method: Method,
    status: StatusCode,
    headers: HeaderMap,
    url: Url,
    text: String,
    html: Html,
    forms: Vec<Form>,
}

impl Page {
    pub(crate) fn build(
        method: Method,
        url: Url,
        status: StatusCode,
        headers: HeaderMap,
        text: String,
    ) -> Self {
        let html = Html::parse_document(&text);
        let forms = Self::parse_forms(&html, &url);

        Self {
            method,
            status,
            headers,
            url,
            text,
            html,
            forms,
        }
    }

    /// Returns the http method used to fetch this page.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns the http status code returned with this page.
    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    /// Returns the response headers returned with this page.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns the url for this page. Due to server-side redirects this url may be different from the initial request.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the unparsed html content of this page.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns a reference to the form at index `idx` from the list of forms on this page.
    pub fn form(&self, idx: usize) -> Result<&Form> {
        self.forms.get(idx).ok_or(Error::FormIndexOutOfBoundsError {
            num_forms: self.forms.len(),
            idx,
        })
    }

    /// Returns a mutable reference to the form at index `idx` from the list of forms on this page.
    pub fn form_mut(&mut self, idx: usize) -> Result<&mut Form> {
        let len = self.forms.len();
        self.forms
            .get_mut(idx)
            .ok_or(Error::FormIndexOutOfBoundsError {
                num_forms: len,
                idx,
            })
    }

    /// Returns a reference to the form with the given `id` from the list of forms on this page.
    pub fn form_by_id(&self, id: &str) -> Result<&Form> {
        for form in &self.forms {
            if form.id().is_some() && form.id().unwrap() == id {
                return Ok(form);
            }
        }

        Err(Error::FormIdNotFoundError { id: id.to_owned() })
    }

    /// Returns a mutable reference to the form with the given `id` from the list of forms on this page.
    pub fn form_by_id_mut(&mut self, id: &str) -> Result<&mut Form> {
        for form in &mut self.forms {
            if form.id().is_some() && form.id().unwrap() == id {
                return Ok(form);
            }
        }

        Err(Error::FormIdNotFoundError { id: id.to_owned() })
    }

    fn parse_selectors(&self, selectors: &str) -> Result<Selector> {
        Selector::parse(selectors).map_err(|error| Error::CssSelectorParseError {
            selector: selectors.to_owned(),
            reason: format!("{error:?}"),
        })
    }

    /// Returns the first element matching the given CSS selector group, i.e. a comma-separated list of selectors. See
    /// [W3Schools: CSS Selector Reference](https://www.w3schools.com/cssref/css_selectors.php).
    ///
    /// ```no_run
    /// # let page: Option<no_browser::page::Page> = None;
    /// # let page = page.unwrap();
    /// let title_element = page.select_first("head > title")?;
    /// let title = title_element.inner_html();
    /// # Ok::<(), no_browser::page::Error>(())
    /// ```
    pub fn select_first(&self, selectors: &str) -> Result<ElementRef> {
        let s = self.parse_selectors(selectors)?;

        self.html
            .select(&s)
            .next()
            .ok_or(Error::CssSelectorResultEmptyError {
                selector: selectors.to_owned(),
            })
    }

    /// Returns all elements matching the given CSS selector group, i.e. a comma-separated list of selectors. See
    /// [W3Schools: CSS Selector Reference](https://www.w3schools.com/cssref/css_selectors.php).
    ///
    /// ```no_run
    /// # let page: Option<no_browser::page::Page> = None;
    /// # let page = page.unwrap();
    /// let elements = page.select("head > title")?;
    /// let last_content = elements.first().unwrap().inner_html();
    /// # Ok::<(), no_browser::page::Error>(())
    /// ```
    pub fn select(&self, selectors: &str) -> Result<Vec<ElementRef>> {
        let selectors = self.parse_selectors(selectors)?;

        Ok(self.html.select(&selectors).collect::<Vec<ElementRef>>())
    }

    /// Returns the value of the query parameter associated with the given name. _Note_: If there are multiple values
    /// associated, only the first hit will be returned!
    pub fn query(&self, name: &str) -> Result<String> {
        for (k, v) in self.url.query_pairs() {
            if k.eq(name) {
                return Ok(v.to_string());
            }
        }

        Err(Error::UnknownQueryParamError {
            query: String::from(self.url.query().unwrap_or("")),
            param: String::from(name),
        })
    }

    fn parse_forms(html: &Html, url: &Url) -> Vec<Form> {
        let mut forms = Vec::new();

        let selector = Selector::parse("form").unwrap();

        for form_ref in html.select(&selector) {
            forms.push(Form::parse(&form_ref, url.clone()));
        }

        forms
    }
}

#[cfg(test)]
mod tests {
    use crate::input::InputType;

    use super::Page;
    use reqwest::{header::HeaderMap, Method, StatusCode, Url};

    static PAGE_001: &str = r#"
        <html>
            <body>
                <h1>Test</h1>
                <form action="subpage1" id="id_01">
                    <input type="hidden" name="hidden" value="hidden">
                    <button type="submit" value="submit" name="submit">Submit</button>
                </form>
                <form action="subpage1" id="id_02">
                    <input type="hidden" name="hidden" value="hidden">
                    <button type="submit" value="submit" name="submit">Submit</button>
                </form>
                <form action="subpage1" id="id_03">
                    <input type="hidden" name="hidden" value="hidden">
                    <button type="submit" value="submit" name="submit">Submit</button>
                </form>
            </body>
        </html>
    "#;

    #[test]
    fn parse_page() {
        let method = Method::GET;
        let url = Url::parse("https://wikipedia.org/").unwrap();
        let status = StatusCode::OK;
        let headers = HeaderMap::new();
        let text = PAGE_001.to_owned();

        let page = Page::build(method, url, status, headers, text);

        assert_eq!(page.method(), Method::GET);
        assert_eq!(*page.status(), StatusCode::OK);
        assert_eq!(page.headers().len(), 0);
        assert_eq!(*page.url(), Url::parse("https://wikipedia.org/").unwrap());
        assert_eq!(page.text(), PAGE_001);

        assert_eq!(page.forms.len(), 3);
        assert_eq!(page.form(0).unwrap().id(), Some("id_01"));
        assert_eq!(page.form(1).unwrap().id(), Some("id_02"));
        assert_eq!(page.form(2).unwrap().id(), Some("id_03"));

        let form = page.form_by_id("id_02").unwrap();
        let hidden = form.input(InputType::Hidden, "hidden").unwrap();
        assert_eq!(hidden.name(), "hidden");
        assert_eq!(hidden.value(), Some("hidden"));
    }
}
