//! Module containing the [`Form`][Form] struct.

use crate::input::{Input, InputType};
use reqwest::{Method, Url};
use scraper::{ElementRef, Html, Selector};
use std::{cell::RefCell, rc::Rc, str::FromStr};
use thiserror::Error;

/// An error occurred while working with the form.
#[derive(Debug, Error)]
pub enum Error {
    /// No input field found for the given `input_name` and `input_type`.
    #[error("Form doesn't contain input named '{input_name}' of type '{input_type:?}'!")]
    InputNotInFormError {
        /// The name of the input to be fetched.
        input_name: String,
        /// The type of the input to be fetched.
        input_type: InputType,
    },
}

/// Short-hand for `std::result::Result<T, no_browser::form::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
/// Short-hand for `Rc<RefCell<no_browser::input::Input>>`.
pub type InputRef = Rc<RefCell<Input>>;

/// Struct [`Form`][Form] represents a parsed html form.
///
/// It gives access to:
/// * this forms id (`id()`);
/// * the individual input fields in this form (`input()`);
///
/// See the main docs of [crate `no_browser`][crate] for usage examples.
#[derive(Debug)]
pub struct Form {
    page_url: Url,
    method: Method,
    action: String,
    id: Option<String>,
    inputs: Vec<InputRef>,
}

pub(crate) struct SubmitFormInfo {
    pub url: String,
    pub method: Method,
    pub data: Vec<(String, String)>,
}

static BUTTONS: [InputType; 3] = [InputType::Button, InputType::Reset, InputType::Submit];

impl Form {
    /// Returns the `id` of this form if it has any.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns a shared reference (`Rc<RefCell<>>`) to an input field ([`Input`][Input]) within this form.
    pub fn input(&self, t: InputType, name: &str) -> Result<InputRef> {
        for input in &self.inputs {
            let input_r = input.borrow();
            if input_r.t() != t || name != input_r.name() {
                continue;
            }

            return Ok(input.clone());
        }

        Err(Error::InputNotInFormError {
            input_name: name.to_owned(),
            input_type: t,
        })
    }

    pub(crate) fn submit(&self, submit_button_name: Option<&str>) -> Result<SubmitFormInfo> {
        let url = self.form_target_url();
        let method = self.method.clone();

        let mut data = Vec::new();

        if let Some(submit_button_name) = submit_button_name {
            let input = self.input(InputType::Submit, submit_button_name)?;
            let button = input.borrow();
            data.push((button.name().to_owned(), button.value().unwrap().to_owned()));
        }

        for input in &self.inputs {
            let input = input.borrow();

            if BUTTONS.contains(&input.t()) {
                continue; // skip buttons
            }
            if input.value().is_none() {
                continue; // skip empty inputs
            }
            if input.t() == InputType::Checkbox && input.attr("checked").is_none() {
                continue; // skip unchecked checkboxes
            }

            data.push((input.name().to_owned(), input.value().unwrap().to_owned()));
        }

        Ok(SubmitFormInfo { url, method, data })
    }

    pub(crate) fn parse(form_ref: &ElementRef, page_url: Url) -> Self {
        let form = form_ref.value();
        let method_s = form.attr("method").unwrap_or("GET");
        let mut method = Method::from_str(&method_s.to_uppercase()).unwrap_or(Method::GET);

        if method != Method::GET && method != Method::POST {
            method = Method::GET;
        }

        let action = form
            .attr("action")
            .or(Some(""))
            .map(|s| s.to_owned())
            .unwrap();
        let id = form.attr("id").map(|s| s.to_owned());

        let inputs = Self::parse_form_inputs(form_ref);
        let inputs = inputs
            .into_iter()
            .map(|input| Rc::new(RefCell::new(input)))
            .collect();

        Self {
            page_url,
            method,
            action,
            id,
            inputs,
        }
    }

    fn parse_form_inputs(form: &ElementRef) -> Vec<Input> {
        let html = Html::parse_fragment(&form.inner_html());
        let mut inputs = Vec::new();

        let selector = Selector::parse("input").unwrap();
        for input in html.select(&selector) {
            let input = input.value();
            if let Ok(input) = Input::parse(input) {
                // Silently drop input parse errors
                inputs.push(input)
            }
        }

        let selector = Selector::parse("button").unwrap();
        for button in html.select(&selector) {
            let button = button.value();
            if let Ok(button) = Input::parse(button) {
                // Silently drop input parse errors
                inputs.push(button)
            }
        }

        inputs
    }

    fn form_target_url(&self) -> String {
        // absolute external action, no work required
        if self.action.starts_with("http://") || self.action.starts_with("https://") {
            return self.action.clone();
        }

        let mut creds = String::from(self.page_url.username());
        if self.page_url.password().is_some() {
            creds.push(':');
            creds.push_str(self.page_url.password().unwrap());
            creds.push('@')
        }

        let mut url = format!(
            "{}://{}{}:{}",
            self.page_url.scheme(),
            creds,
            self.page_url.host_str().unwrap_or(""),
            self.page_url.port_or_known_default().unwrap(),
        );

        if !self.action.starts_with('/') {
            // action relative to the current path; so add current path
            if self.page_url.path().ends_with('/') {
                // discard trailing slash of current path
                url.push_str(&self.page_url.path()[..self.page_url.path().len() - 1]);
            } else {
                // discard last page / file
                let mut path_parts: Vec<&str> = self.page_url.path().split('/').collect();
                path_parts.pop();
                url.push_str(&path_parts.join("/"));
            }
        }

        url.push_str(&self.action);

        url
    }
}

#[cfg(test)]
mod tests {
    use reqwest::{Method, Url};
    use scraper::{Html, Selector};

    use crate::input::InputType;

    use super::{Form, Result};

    static FORM_001: &str = r#"
    <html>
        <body>
            <form id="form_01" method="GET" action="https://www.github.com/submit_stuff">
                <input name="txt" type="text" value="txt">
                <input name="chk_a" type="checkbox" value="chk_a" checked>
                <input name="chk_b" type="checkbox" value="chk_b">
                <button name="ok" type="submit" value="ok">OK</button>
            </form>
        </body>
    </html>"#;

    #[test]
    fn parse_form() -> Result<()> {
        let html = Html::parse_fragment(FORM_001);
        let selector = Selector::parse("form").unwrap();
        let form = html.select(&selector).into_iter().next().unwrap();

        let form = Form::parse(&form, Url::parse("https://wikipedia.org/").unwrap());

        assert_eq!(form.page_url, Url::parse("https://wikipedia.org/").unwrap());
        assert_eq!(form.method, Method::GET);
        assert_eq!(form.action, "https://www.github.com/submit_stuff");
        assert_eq!(form.inputs.len(), 4);

        assert_eq!(
            form.form_target_url(),
            "https://www.github.com/submit_stuff"
        );

        Ok(())
    }

    #[test]
    fn submit_checkboxes() -> Result<()> {
        let html = Html::parse_fragment(FORM_001);
        let selector = Selector::parse("form").unwrap();
        let form = html.select(&selector).into_iter().next().unwrap();

        let form = Form::parse(&form, Url::parse("https://wikipedia.org/").unwrap());

        let info = form.submit(Some("ok"))?;
        assert_eq!(info.method, Method::GET);
        assert_eq!(info.url, "https://www.github.com/submit_stuff");
        assert_eq!(info.data.len(), 3);

        assert!(info.data.contains(&("txt".to_owned(), "txt".to_owned())));
        assert!(info.data.contains(&("ok".to_owned(), "ok".to_owned())));
        assert!(info
            .data
            .contains(&("chk_a".to_owned(), "chk_a".to_owned())));
        assert!(!info
            .data
            .contains(&("chk_b".to_owned(), "chk_b".to_owned())));

        // Check second checkbox
        form.input(InputType::Checkbox, "chk_b")?
            .borrow_mut()
            .set_attr("checked", Some("".to_owned()));

        let info = form.submit(Some("ok"))?;
        assert_eq!(info.method, Method::GET);
        assert_eq!(info.url, "https://www.github.com/submit_stuff");
        assert_eq!(info.data.len(), 4);

        assert!(info.data.contains(&("txt".to_owned(), "txt".to_owned())));
        assert!(info.data.contains(&("ok".to_owned(), "ok".to_owned())));
        assert!(info
            .data
            .contains(&("chk_a".to_owned(), "chk_a".to_owned())));
        assert!(info
            .data
            .contains(&("chk_b".to_owned(), "chk_b".to_owned())));

        // uncheck both checkboxes
        form.input(InputType::Checkbox, "chk_a")?
            .borrow_mut()
            .set_attr("checked", None);
        form.input(InputType::Checkbox, "chk_b")?
            .borrow_mut()
            .set_attr("checked", None);

        let info = form.submit(Some("ok"))?;
        assert_eq!(info.method, Method::GET);
        assert_eq!(info.url, "https://www.github.com/submit_stuff");
        assert_eq!(info.data.len(), 2);

        assert!(info.data.contains(&("txt".to_owned(), "txt".to_owned())));
        assert!(info.data.contains(&("ok".to_owned(), "ok".to_owned())));

        Ok(())
    }
}
