//! HTTP-related types and constants

use gpui::{AnyElement, App, IntoElement, SharedString, Window};
use gpui_kit::component::select::SelectItem;
use serde::{Deserialize, Serialize};

use crate::ui::method_badge::MethodBadge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Json,
    Xml,
    Text,
    Html,
    Form,
    UrlEncoded,
}

impl HttpMethod {
    pub const ALL: [HttpMethod; 7] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
        HttpMethod::Options,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }

    /// Compact label for badges, at most five characters.
    pub fn short_str(&self) -> &'static str {
        match self {
            HttpMethod::Delete => "DEL",
            HttpMethod::Options => "OPT",
            other => other.as_str(),
        }
    }
}

impl ContentType {
    pub const ALL: [ContentType; 6] = [
        ContentType::Json,
        ContentType::Xml,
        ContentType::Text,
        ContentType::Html,
        ContentType::Form,
        ContentType::UrlEncoded,
    ];

    pub fn from_header(content_type: &str) -> Self {
        let content_type = content_type.to_lowercase();
        if content_type.contains("application/json") {
            ContentType::Json
        } else if content_type.contains("application/xml") || content_type.contains("text/xml") {
            ContentType::Xml
        } else if content_type.contains("text/html") {
            ContentType::Html
        } else if content_type.contains("text/plain") {
            ContentType::Text
        } else if content_type.contains("application/x-www-form-urlencoded") {
            ContentType::Form
        } else {
            ContentType::Json // Default
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Json => "application/json",
            ContentType::Xml => "application/xml",
            ContentType::Text => "text/plain",
            ContentType::Html => "text/html",
            ContentType::Form => "application/x-www-form-urlencoded",
            ContentType::UrlEncoded => "application/x-www-form-urlencoded",
        }
    }

    pub fn body_type(&self) -> &'static str {
        match self {
            ContentType::Json => "json",
            ContentType::Xml => "xml",
            ContentType::Text => "text",
            ContentType::Html => "html",
            ContentType::Form => "form",
            ContentType::UrlEncoded => "form",
        }
    }

    pub fn language(&self) -> &'static str {
        match self {
            ContentType::Json => "json",
            ContentType::Xml => "xml",
            ContentType::Text => "text",
            ContentType::Html => "html",
            ContentType::Form => "text",
            ContentType::UrlEncoded => "text",
        }
    }
}

impl SelectItem for ContentType {
    type Value = ContentType;

    fn title(&self) -> SharedString {
        match self {
            ContentType::Json => "JSON".into(),
            ContentType::Xml => "XML".into(),
            ContentType::Text => "Plain Text".into(),
            ContentType::Html => "HTML".into(),
            ContentType::Form => "Form Data".into(),
            ContentType::UrlEncoded => "URL Encoded".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl SelectItem for HttpMethod {
    type Value = HttpMethod;

    fn title(&self) -> SharedString {
        self.as_str().into()
    }

    fn display_title(&self) -> Option<AnyElement> {
        Some(MethodBadge::new(*self).into_any_element())
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        MethodBadge::new(*self)
    }

    fn value(&self) -> &Self::Value {
        self
    }
}
