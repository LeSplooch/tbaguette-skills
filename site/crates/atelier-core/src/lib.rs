//! Shared model and renderers for the TBaguette site: the same code builds
//! the pages on the host and (later) runs in the browser as WASM.

pub mod content;
pub mod escape;
pub mod frontmatter;
pub mod graph;
pub mod highlight;
pub mod markdown;
pub mod model;
pub mod router;
pub mod updates;
