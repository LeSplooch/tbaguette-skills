//! The two HTML escapers the Python build used. They are deliberately
//! different, and mixing them changes the pages' bytes: the apostrophe is the
//! only difference.

/// `templates.escape_html`, i.e. Python's `html.escape(s, quote=True)`:
/// `& < > " '` become `&amp; &lt; &gt; &quot; &#x27;`. Used for everything
/// the page templates and the syntax highlighter interpolate.
pub fn html_attr(text: &str) -> String {
    escape(text, true)
}

/// `content_pipeline.escape_html`: `& < > "` only. The apostrophe stays raw.
/// Used for text the Markdown renderer emits.
pub fn html_text_pipeline(text: &str) -> String {
    escape(text, false)
}

fn escape(text: &str, escape_apostrophe: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' if escape_apostrophe => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}
