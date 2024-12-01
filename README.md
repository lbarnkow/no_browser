# no_browser

The `no_browser` crate strives to provide a high-level API for programmatically interacting with web pages through
a _light-weight, head-less "web browser"_. This is not a real web browser, like Firefox or Chrome.

`no_browser` builds on top of [reqwest](https://crates.io/crates/reqwest) handling http requests, http redirects and
cookie management, as well as, [scraper](https://crates.io/crates/scraper) parsing html code. This crate uses
[CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors) to freely access any element of a
given web page. It also provides its own abstraction to fill out and submit web forms.

However, `no_browser` has no support for client-side JavaScript or any concept of actually rendering web pages. So if
you want to test modern JavaScript-driven web apps, opt for a crate driving a real web browser like
[fantoccini](https://crates.io/crates/fantoccini) or [thirtyfour](https://crates.io/crates/thirtyfour), instead.

## Example

```rust,no_run
let browser = Browser::builder().finish()?;

// Lets go to the Wikipedia main page
let mut page = browser.navigate_to("https://en.wikipedia.org/", None)?;

// the title tag should be "Wikipedia, the free encyclopedia"
assert_eq!(
    page.select_first("head > title")?.inner_html(),
    "Wikipedia, the free encyclopedia"
);

// the main page should welcome us
assert!(page
    .select_first("div.mw-heading > h1")?
    .inner_html()
    .starts_with("Welcome to"));

// fill out the search form ...
let search_form = page.form_by_id_mut("searchform")?;
search_form
    .input_mut(InputType::Search, "search")?
    .set_value(Some("rust programming language".to_owned()));

// ... and submit
let page = browser.submit_form(search_form, None)?;

// the new title tag should be "Rust (programming language) - Wikipedia"
assert_eq!(
    page.select_first("head > title")?.inner_html(),
    "Rust (programming language) - Wikipedia"
);

// The main title on the html page should be quite similar
assert_eq!(
    page.select_first("span.mw-page-title-main")?.inner_html(),
    "Rust (programming language)"
);

// The main title should have a subtitle showing we've been redirected
assert!(page
    .select_first("div#contentSub span.mw-redirectedfrom")?
    .inner_html()
    .starts_with("(Redirected from"));

// The table of contents should have more than 10 entries
assert!(page
    .select("div#vector-toc ul#mw-panel-toc-list li.vector-toc-list-item")?
    .len() > 10);
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

`SPDX-License-Identifier: MIT OR Apache-2.0`

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
