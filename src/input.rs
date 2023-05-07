//! Module containing the [`Input`][Input] struct.

use lazy_static::lazy_static;
use scraper::node::Element;
use std::collections::HashMap;
use thiserror::Error;

/// An error occurred while parsing the input element or while working with it.
#[derive(Debug, Error)]
pub enum Error {
    /// Form inputs without a name attribute are not supported.
    #[error("Unnamed inputs are not supported!")]
    UnnamedInputError {},

    /// Only `<input>` and `<button>` elements can be parsed.
    #[error("Html tag '{element_tag}' cannot be parsed to struct Input!")]
    UnsupportedElementTagError {
        /// The actual unparsable element tag.
        element_tag: String,
    },

    /// Not all legal html input types may be supported by [`no_browser`][crate].
    #[error("Input tag with attribte 'type={attr_type}' cannot be parsed to struct Input!")]
    UnsupportedInputTypeError {
        /// The unsupported input `type`.
        attr_type: String,
    },

    /// The input can't be parsed because a mandatory attribute is missing.
    #[error("Missing attribute '{attribute}' on html tag '{element_tag}'!")]
    MissingAttributeError {
        /// The missing attribute name.
        attribute: String,
        /// The element tag missing the attribute.
        element_tag: String,
    },
}

/// Short-hand for `std::result::Result<T, no_browser::input::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// The supported html input elements.
///
/// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputType {
    /// see <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/button> <br/>
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button>
    Button,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/checkbox>
    Checkbox,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/color>
    Color,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/date>
    Date,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/datetime-local>
    DateTimeLocal,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/email>
    Email,
    // See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/file>
    // File,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/hidden>
    Hidden,
    // See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/image>
    // Image,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/month>
    Month,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/number>
    Number,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/password>
    Password,
    // See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/radio>
    // Radio,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/range>
    Range,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/reset> <br/>
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button>
    Reset,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/search>
    Search,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/submit> <br/>
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button>
    Submit,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/tel>
    Tel,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/text>
    Text,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/time>
    Time,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/url>
    Url,
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input/week>
    Week,
}

lazy_static! {
    static ref MAPPINGS: HashMap<&'static str, InputType> = {
        HashMap::from([
            ("button", InputType::Button),
            ("checkbox", InputType::Checkbox),
            ("color", InputType::Color),
            ("date", InputType::Date),
            ("datetime-local", InputType::DateTimeLocal),
            ("email", InputType::Email),
            // ("file", InputType::File),
            ("hidden", InputType::Hidden),
            // ("image", InputType::Image),
            ("month", InputType::Month),
            ("number", InputType::Number),
            ("password", InputType::Password),
            // ("radio", InputType::Radio),
            ("range", InputType::Range),
            ("reset", InputType::Reset),
            ("search", InputType::Search),
            ("submit", InputType::Submit),
            ("tel", InputType::Tel),
            ("text", InputType::Text),
            ("time", InputType::Time),
            ("url", InputType::Url),
            ("week", InputType::Week),
        ])
    };
}

/// Struct [`Input`][Input] represents a parsed html form input element.
///
/// It gives access to:
/// * this input's type (`t()`);
/// * this input's name (`name()`);
/// * this input's value (`value()` / `set_value()`);
/// * this input's other attributes (`attr()` / `set_attr()`);
///
/// See the main docs of [crate `no_browser`][crate] for usage examples.
#[derive(Debug)]
pub struct Input {
    t: InputType,
    name: String,
    value: Option<String>,
    attr: HashMap<String, String>,
}

impl Input {
    /// Returns the [`InputType`][InputType] of this input element.
    pub fn t(&self) -> InputType {
        self.t
    }

    /// Returns the `name` attribute of this input element.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the `value` attribute of this input element.
    pub fn value(&self) -> Option<&str> {
        self.value.as_ref().map(|s| s.as_str())
    }

    /// Sets the `value` attribute of this input element.
    pub fn set_value(&mut self, new_value: Option<String>) -> Option<String> {
        let prev = self.value.take();
        self.value = new_value;
        prev
    }

    /// Returns the value associated with the given attribute name.
    pub fn attr(&self, attr: &str) -> Option<&str> {
        self.attr.get(attr).map(|s| s.as_str())
    }

    /// Sets the value associated with the given attribute name.
    pub fn set_attr(&mut self, attr: &str, new_value: Option<String>) -> Option<String> {
        let prev;

        if new_value.is_none() {
            prev = self.attr.remove(attr)
        } else {
            prev = self.attr.remove(attr);
            self.attr.insert(attr.to_owned(), new_value.unwrap());
        }
        prev
    }

    pub(crate) fn parse(element: &Element) -> Result<Input> {
        let tag_name = element.name().to_lowercase();

        match tag_name.as_str() {
            "input" => Self::parse_input(element),
            "button" => Self::parse_button(element),
            _ => Err(Error::UnsupportedElementTagError {
                element_tag: tag_name,
            }),
        }
    }

    fn parse_input(element: &Element) -> Result<Input> {
        let t = element
            .attr("type")
            .ok_or_else(|| Error::MissingAttributeError {
                attribute: "type".to_owned(),
                element_tag: element.name().to_owned(),
            })?;

        let t = MAPPINGS
            .get(t)
            .ok_or_else(|| Error::UnsupportedInputTypeError {
                attr_type: t.to_owned(),
            })?
            .to_owned();

        Self::parse_element(element, t)
    }

    fn parse_button(element: &Element) -> Result<Input> {
        let t = element
            .attr("type")
            .or(Some("submit"))
            .unwrap()
            .to_lowercase();

        let t = match t.as_str() {
            "submit" => InputType::Submit,
            "reset" => InputType::Reset,
            "button" => InputType::Button,
            _ => return Err(Error::UnsupportedInputTypeError { attr_type: t }),
        };

        Self::parse_element(element, t)
    }

    fn parse_element(element: &Element, t: InputType) -> Result<Input> {
        let name = element
            .attr("name")
            .ok_or_else(|| Error::UnnamedInputError {})?
            .to_owned();
        let value = element.attr("value").map(|s| s.to_owned());

        let mut attr = HashMap::new();
        for (k, v) in element.attrs() {
            attr.insert(k.to_owned(), v.to_owned());
        }

        Ok(Input {
            t,
            name,
            value,
            attr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Input, InputType, Result};
    use rstest::rstest;
    use scraper::{Html, Selector};

    #[rstest]
    #[case("button", InputType::Button)]
    #[case("checkbox", InputType::Checkbox)]
    #[case("color", InputType::Color)]
    #[case("date", InputType::Date)]
    #[case("datetime-local", InputType::DateTimeLocal)]
    #[case("email", InputType::Email)]
    // #[case("file", InputType::File)]
    #[case("hidden", InputType::Hidden)]
    // #[case("image", InputType::Image)]
    #[case("month", InputType::Month)]
    #[case("number", InputType::Number)]
    #[case("password", InputType::Password)]
    // #[case("radio", InputType::Radio)]
    #[case("range", InputType::Range)]
    #[case("reset", InputType::Reset)]
    #[case("search", InputType::Search)]
    #[case("submit", InputType::Submit)]
    #[case("tel", InputType::Tel)]
    #[case("text", InputType::Text)]
    #[case("time", InputType::Time)]
    #[case("url", InputType::Url)]
    #[case("week", InputType::Week)]
    fn parse_valid_inputs(
        #[case] input_type: &str,
        #[case] expected_type: InputType,
    ) -> Result<()> {
        let raw_html = format!(
            r#"<input class="the_class" name="the_{t}" type="{t}" value="the_value" k1="v1" k2="v2">"#,
            t = input_type
        );
        let html = Html::parse_fragment(&raw_html);
        let selector = Selector::parse("input").unwrap();
        let element = html.select(&selector).next().unwrap();

        let mut input = Input::parse(element.value())?;

        assert_eq!(input.t(), expected_type);
        assert_eq!(input.name(), format!("the_{input_type}"));
        assert_eq!(input.value(), Some("the_value"));

        assert_eq!(input.attr("k1"), Some("v1"));
        assert_eq!(input.attr("k2"), Some("v2"));
        assert_eq!(input.attr("k3"), None);

        input.set_value(Some("new_value".to_owned()));
        assert_eq!(input.value(), Some("new_value"));
        input.set_value(None);
        assert_eq!(input.value(), None);

        input.set_attr("k1", Some("v1_new".to_owned()));
        assert_eq!(input.attr("k1"), Some("v1_new"));

        Ok(())
    }
}
