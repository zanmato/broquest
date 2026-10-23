//! Collection management module

mod collection_editor;
mod format;
mod group_editor;
mod http_files;
mod manager;
mod openapi;
mod opencollection;
mod panel;
mod storage;
mod types;
mod vars_view;
mod wsdl;

pub use collection_editor::*;
pub use format::*;
pub use group_editor::*;
pub use http_files::is_http_collection;
pub use manager::*;
pub use opencollection::*;
pub use panel::*;
pub use types::*;
pub use vars_view::*;
