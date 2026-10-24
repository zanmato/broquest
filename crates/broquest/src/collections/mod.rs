//! Collection management module

pub(crate) mod collection_editor;
mod format;
pub(crate) mod group_editor;
mod http_files;
mod http_save;
mod http_watch;
mod manager;
mod openapi;
mod opencollection;
mod panel;
mod storage;
mod types;
mod vars_view;
mod wsdl;

pub(crate) use collection_editor::CONTEXT as COLLECTION_EDITOR_CONTEXT;
pub use collection_editor::*;
pub use format::*;
pub(crate) use group_editor::CONTEXT as GROUP_EDITOR_CONTEXT;
pub use group_editor::*;
pub use http_files::is_http_collection;
pub use http_save::DEFAULT_FILE as HTTP_DEFAULT_FILE;
pub use manager::*;
pub use opencollection::*;
pub use panel::*;
pub use types::*;
pub use vars_view::*;
