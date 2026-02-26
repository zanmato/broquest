// Application Events

use crate::{
    collections::CollectionToml,
    domain::{RequestData, ResponseData},
};
use gpui::SharedString;

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
        collection_path: SharedString,
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
        collection_path: SharedString,
    },
    CollectionSaved {
        collection_path: SharedString,
        collection_name: String,
        success: bool,
    },
    CollectionDeleted {
        collection_path: SharedString,
    },
    NewRequest {
        collection_path: SharedString,
        group_path: Option<SharedString>,
    },
    GroupCreated {
        collection_path: SharedString,
        group_name: SharedString,
    },
    GroupDeleted {
        collection_path: SharedString,
        group_name: SharedString,
    },
    /// Create a new group tab
    CreateNewGroupTab {
        collection_path: SharedString,
        group_name: Option<SharedString>,
    },
    /// Request was moved (drag and drop)
    RequestMoved,
}

/// Error severity levels
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
