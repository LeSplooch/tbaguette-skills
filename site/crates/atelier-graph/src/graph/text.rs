//! The graph's copy (graph.js `STR`, one table so the voice stays one voice)
//! and the string helpers that reproduce JavaScript's own formatting: `fmt`'s
//! `{key}` templates, `escapeHtml`, `Number.prototype.toString`, `Math.round`,
//! `toFixed` and `toLocaleString('en')`.

pub const TITLE: &str = "The crumb";
pub const META: &str = "{skills} skills · {pairs} cross-references";
pub const VIEWS_LABEL: &str = "View";
pub const SEARCH_LABEL: &str = "Find a skill";
pub const SEARCH_PLACEHOLDER: &str = "Find a skill…";
pub const ASK_HITS_ONE: &str = "1 skill mentions “{q}”: {names}.";
pub const ASK_HITS: &str = "{n} skills mention “{q}”: {names}.";
pub const ASK_HITS_MORE: &str = "{n} skills mention “{q}”: {names} and more.";
pub const ASK_MISS: &str = "No skill mentions “{q}”.";
pub const TRACE: &str = "Trace a path";
pub const LIBRARY: &str = "Library";
pub const BACK: &str = "Back";
pub const ZOOM_IN: &str = "Zoom in";
pub const ZOOM_OUT: &str = "Zoom out";
pub const FIT: &str = "Fit to view";
pub const SIZE_BY: &str = "Size by";
pub const STAGE_LABEL: &str =
    "Skill graph. Arrow keys move between skills, Enter opens one, Escape steps back. The List view has the same graph as text.";
pub const LOADING: &str = "Kneading the crumb…";
pub const FAILED: &str = "The graph could not load.";
pub const RETRY: &str = "Try again";
pub const HINT: &str = "Drag to pan · scroll or pinch to zoom · click a skill to open it · or use the arrow keys and Enter";
pub const HINT_TOUCH: &str = "Dot = skill · line = citation · bigger = cited more. Drag to pan, pinch to zoom, tap a skill, then tap again to open it";
pub const ANATOMY_HINT: &str =
    "Each arc is a section, sized by its length. Click a skill around it to walk there.";
pub const INTRO_TITLE: &str = "How the library holds together";
pub const INTRO: &str = "Each dot is a skill and each line a citation, drawn from the skill that makes it toward the one it names. Skills gather by family; size shows how often a skill is cited.";
pub const NOTES_TITLE: &str = "Worth noticing";
pub const ROLES_TITLE: &str = "Every skill’s role";
pub const FAMILIES_TITLE: &str = "Families";
pub const ALWAYS_ON: &str = "Runs in every conversation";
pub const FRESH_NEW: &str = "New in the last two days";
pub const FRESH_UPDATED: &str = "Updated in the last two days";
pub const CITED_BY: &str = "Cited by";
pub const CITES: &str = "Cites";
pub const SECTIONS: &str = "Sections";
pub const WORDS: &str = "words";
pub const WORDS_COUNT: &str = "{n} words";
pub const CITES_COUNT: &str = "cites {n}";
pub const CITED_BY_COUNT: &str = "cited by {n}";
pub const OPEN_SKILL: &str = "Open the skill page";
pub const TRACE_FROM: &str = "Trace a path from here";
pub const REACH: &str = "Two steps of citations take it to <b>{n} skills</b> in {f}.";
pub const REACH_FAMILIES: [&str; 2] = ["{k} family", "{k} families"];
pub const REACH_LIGHT: &str = "Light them up";
pub const REACH_SHOWN: &str = "{name} and the {n} skills within two citations, shown in the graph.";
pub const READ_SECTION: &str = "Read this section";
pub const TRIGGER: &str = "When to use";
pub const TRIGGER_NOTE: &str =
    "Named in its trigger description, the part an agent always has loaded.";
pub const REFERENCE: &str = "Reference";
pub const FROM_SECTION: &str = "from “{title}”";
pub const INBOUND_FROM: &str = "Cites it from “{title}”";
pub const NO_CITES: &str = "Cites nothing from here.";
pub const TRACE_TITLE: &str = "Trace a path";
pub const TRACE_INTRO: &str = "Follow citations from one skill to another, the way a reader clicking through the pages would.";
pub const TRACE_FROM_LABEL: &str = "From";
pub const TRACE_TO_LABEL: &str = "To";
pub const TRACE_GO: &str = "Find the path";
pub const TRACE_SURPRISE: &str = "Surprise me";
pub const TRACE_NONE: &str = "No route follows the citations in that direction.";
pub const TRACE_SAME: &str = "Pick two different skills.";
pub const TRACE_UNKNOWN: &str = "No skill by that name.";
pub const TRACE_STEPS: &str = "{n} steps";
pub const TRACE_STEP: &str = "1 step";
pub const TRACE_DONE: &str = "Done";
pub const COPY_LINK: &str = "Copy link to this walk";
pub const COPIED_LINK: &str = "Link copied";
pub const LEGEND_CITES: &str = "it cites";
pub const LEGEND_CITED_BY: &str = "cites it";
pub const LEGEND_BOTH: &str = "both ways";
pub const LIST_INTRO: &str = "Every skill, what it cites, and what cites it.";
pub const LIST_OPEN: &str = "Open in the graph";
pub const LIST_JUMP: &str = "Jump to a family";
pub const MATRIX_CAPTION: &str = "Citations between families. Rows cite, columns are cited.";
pub const MATRIX_INTRO: &str = "How the families lean on each other. Read a row across for what that family cites, a column down for who cites it. Each number counts pairs of skills, so the whole grid adds up to the {pairs} cross-references; the outlined diagonal is each family citing itself. Choose a number to see those citations in the graph, or a family to mark its row and column.";
pub const MATRIX_CORNER: [&str; 2] = ["Cites ↓", "Cited →"];
pub const MATRIX_STRONGEST: &str = "Across families, the strongest pull is {a} citing {b}, {n} times; the family most cited from outside itself is {c}.";
pub const MATRIX_FAMILY_OUT: &str = "{a} cites {b} most, {n} times";
pub const MATRIX_FAMILY_IN: &str = "{a} is cited most by {b}, {n} times";
pub const MATRIX_FAMILY_IN_TOO: &str = "and is cited most by {b}, {n} times";
pub const MATRIX_FAMILY_NONE: &str = "{a} has no citations to or from other families";
pub const MATRIX_CELL: &str = "{a} cites {b}: {n}";
pub const MATRIX_CELL_SELF: &str = "{a} cites itself: {n}";
pub const MATRIX_NONE: &str = "{a} never cites {b}";
pub const MATRIX_SHOWN: &str = "{a} citing {b}: {n}, shown in the graph.";
pub const MATRIX_ROW_ON: &str = "{a} marked in the matrix.";
pub const MATRIX_ROW_OFF: &str = "{a} no longer marked.";
pub const ANNOUNCE_VIEW: &str = "{view} view, {n} skills in {f} families.";
pub const ANNOUNCE_TRACE: &str = "Path from {a} to {b}, {n} steps: {names}.";
pub const ANNOUNCE_NO_TRACE: &str = "No path from {a} to {b}.";
pub const ANNOUNCE_NODE: &str = "{name}, {family}. {role}. Cited by {in}, cites {out}.";
pub const ANNOUNCE_SECTION: &str = "Section {n}: {title}. {words} words. Cites {cites}.";

/// The view tabs' names, by view.
pub fn view_name(view: &str) -> &'static str {
    match view {
        "constellation" => "Constellation",
        "wheel" => "Wheel",
        "matrix" => "Matrix",
        "list" => "List",
        _ => "",
    }
}

pub fn size_name(key: &str) -> &'static str {
    match key {
        "in" => "Cited by",
        "out" => "Cites",
        _ => "Length",
    }
}

/// A role's (name, blurb).
pub fn role_copy(role: &str) -> (&'static str, &'static str) {
    match role {
        "foundation" => ("Foundation", "Many skills lean on it"),
        "signpost" => ("Signpost", "Points the way to many others"),
        "crossroads" => ("Crossroads", "Leans on many, and many lean on it"),
        _ => ("Specialist", "A focused craft with a few close ties"),
    }
}

pub struct Notes {
    pub reach: &'static str,
    pub reach_partial: &'static str,
    pub most_cited: &'static str,
    pub widest: &'static str,
    pub widest_all: &'static str,
    pub bridge: &'static str,
    pub bond: &'static str,
    pub family: &'static str,
    pub mutual: &'static str,
    pub quiet: &'static str,
    pub quiet_pair: &'static str,
    pub quiet_many: &'static str,
    pub thin: &'static str,
}

pub const STR_NOTES: Notes = Notes {
    reach: "<b>Every skill reaches every other</b> by following citations — never more than {d} steps. The longest walk runs from {a} to {b}.{ties}",
    reach_partial: "Following citations, {pct}% of all pairs of skills are connected. The longest walk runs from {a} to {b}, {d} steps.{ties}",
    most_cited: "{a} is cited by <b>{n} skills</b>, more than any other.",
    widest: "{a} points to <b>{n} of the other {total}</b>.",
    widest_all: "{a} points to <b>every other skill</b>, all {total} of them.",
    bridge: "{a} is the bridge: more of the shortest routes between skills run through it than through any other.",
    bond: "{a} and {b} cite each other <b>{n} times</b> — the strongest bond here.",
    family: "{a} keeps <b>{p}%</b> of its citations in the family; {b} sends <b>{q}%</b> of its outward.",
    mutual: "<b>{n} pairs</b> of skills cite each other.",
    quiet: "{a} keeps to itself: cited by {i}, cites {o}.",
    quiet_pair: "{a} and {b} keep to themselves: each is cited by {i} and cites {o}.",
    quiet_many: "{a} and <b>{k} others</b> keep to themselves: each is cited by {i} and cites {o}.",
    thin: "<b>{n} skills</b> are cited by {k} others or fewer, so a reader rarely arrives at them by following a citation.",
};

/// graph.js `fmt`: every `{word}` with a value replaced, once, left to right;
/// unknown keys stay as written.
pub fn fmt(template: &str, values: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let key_len = after
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        if key_len > 0 && after.as_bytes().get(key_len) == Some(&b'}') {
            let key = &after[..key_len];
            match values.iter().find(|(k, _)| *k == key) {
                Some((_, v)) => out.push_str(v),
                None => {
                    out.push('{');
                    out.push_str(key);
                    out.push('}');
                }
            }
            rest = &after[key_len + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

pub fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// `String(number)`: shortest round-trip digits, exponent form below 1e-6
/// and from 1e21, and no negative zero.
pub fn num(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if x == 0.0 {
        return "0".into();
    }
    let a = x.abs();
    if (1e-6..1e21).contains(&a) {
        return format!("{x}");
    }
    // Rust writes 1e21 and 1.5e-7; JavaScript writes 1e+21 and 1.5e-7.
    let s = format!("{x:e}");
    match s.split_once('e') {
        Some((m, e)) if !e.starts_with('-') => format!("{m}e+{e}"),
        _ => s,
    }
}

/// `Math.round`: halves toward +∞.
pub fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 { f + 1.0 } else { f }
}

/// `Number.prototype.toFixed(3)` for a value in 0..=1: an exact tie rounds up.
pub fn to_fixed3(x: f64) -> String {
    let scaled = x * 1000.0;
    if scaled.fract() == 0.5 && scaled / 1000.0 == x {
        return format!("{:.3}", (scaled + 0.5) / 1000.0);
    }
    format!("{x:.3}")
}

/// `toLocaleString('en')` for a whole number: thousands separated by commas.
pub fn grouped(n: f64) -> String {
    let digits = format!("{}", n.abs().round() as u64);
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0.0 { format!("-{out}") } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_replaces_known_keys_once_and_keeps_unknown_ones() {
        assert_eq!(
            fmt("{a} and {b} {c}", &[("a", "{b}".into()), ("b", "2".into())]),
            "{b} and 2 {c}"
        );
        assert_eq!(fmt("x { y {}", &[]), "x { y {}");
    }

    #[test]
    fn numbers_print_as_javascript_prints_them() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(-0.0), "0");
        assert_eq!(num(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(num(1e21), "1e+21");
        assert_eq!(num(1.5e-7), "1.5e-7");
        assert_eq!(num(787.5), "787.5");
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(to_fixed3(0.0625), "0.063");
        assert_eq!(to_fixed3(1.0), "1.000");
        assert_eq!(grouped(12345.0), "12,345");
        assert_eq!(grouped(999.0), "999");
    }
}
