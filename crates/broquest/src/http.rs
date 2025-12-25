//! HTTP-related types and constants

use gpui::{App, SharedString};
use gpui_component::{select::SelectItem, ActiveTheme};
use serde::{Deserialize, Serialize};

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

    /// Get color for HTTP method
    pub fn get_color(&self, cx: &App) -> gpui::Hsla {
        match self {
            HttpMethod::Get => cx.theme().green,
            HttpMethod::Post => cx.theme().blue,
            HttpMethod::Put => cx.theme().yellow,
            HttpMethod::Delete => cx.theme().red,
            HttpMethod::Patch => cx.theme().yellow,
            HttpMethod::Head => cx.theme().blue,
            HttpMethod::Options => cx.theme().cyan,
        }
    }

    pub fn get_color_fn(&self) -> fn(cx: &App) -> gpui::Hsla {
        match self {
            HttpMethod::Get => |cx| cx.theme().green,
            HttpMethod::Post => |cx| cx.theme().blue,
            HttpMethod::Put => |cx| cx.theme().yellow,
            HttpMethod::Delete => |cx| cx.theme().red,
            HttpMethod::Patch => |cx| cx.theme().yellow,
            HttpMethod::Head => |cx| cx.theme().blue,
            HttpMethod::Options => |cx| cx.theme().cyan,
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

    fn value(&self) -> &Self::Value {
        self
    }
}
