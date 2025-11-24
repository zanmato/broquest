// Application Events

use crate::{
    collection_types::CollectionToml,
    request_editor::{RequestData, ResponseData},
};

/// Core application events that can be emitted and subscribed to
#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    /// UI events
    ToggleSidebar,

    /// UI events
    ThemeChanged(gpui_component::ThemeMode),
    SidebarToggled {
        collapsed: bool,
    },
    TabChanged {
        tab_id: usize,
    },

    /// Error events
    ErrorOccurred {
        context: String,
        error: String,
        severity: ErrorSeverity,
    },

    /// File operation events
    FileSaved {
        file_path: String,
        success: bool,
    },
    FileOpened {
        file_path: String,
    },

    /// HTTP Request events
    SendRequest(RequestData),
    SaveRequest(RequestData),
    SaveRequestAs(RequestData),
    CreateNewRequestTab {
        request_data: RequestData,
        collection_id: Option<i64>,
    },
    RequestCompleted {
        request_data: RequestData,
        response_data: ResponseData,
    },
    RequestFailed {
        request_data: RequestData,
        error: String,
    },

    /// Collection events
    CreateNewCollectionTab {
        collection_data: CollectionToml,
        collection_path: String,
        collection_id: Option<i64>,
    },
    CollectionSaved {
        collection_path: String,
        collection_name: String,
        success: bool,
    },
    CollectionDeleted {
        collection_id: i64,
    },
    NewRequest {
        collection_id: Option<i64>,
    },
}

/// Error severity levels
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
