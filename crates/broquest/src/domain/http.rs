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
    /// A multipart body, edited as text.
    Multipart,
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
    pub const ALL: [ContentType; 7] = [
        ContentType::Json,
        ContentType::Xml,
        ContentType::Text,
        ContentType::Html,
        ContentType::Form,
        ContentType::UrlEncoded,
        ContentType::Multipart,
    ];

    pub fn from_header(content_type: &str) -> Self {
        let content_type = content_type.to_lowercase();
        let mime = content_type.split(';').next().unwrap_or("").trim();
        if mime == "application/json" || mime.ends_with("+json") {
            ContentType::Json
        } else if mime == "application/xml" || mime == "text/xml" || mime.ends_with("+xml") {
            ContentType::Xml
        } else if mime == "text/html" {
            ContentType::Html
        } else if mime == "application/x-www-form-urlencoded" {
            ContentType::Form
        } else if mime.starts_with("multipart/") {
            ContentType::Multipart
        } else if mime.starts_with("text/") {
            ContentType::Text
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
            ContentType::Multipart => "multipart/form-data",
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
            ContentType::Multipart => "text",
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
            ContentType::Multipart => "text",
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
            ContentType::Multipart => "Multipart".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_types_from_headers() {
        let cases = [
            ("application/json; charset=utf-8", ContentType::Json),
            ("application/problem+json", ContentType::Json),
            ("application/atom+xml", ContentType::Xml),
            ("text/xml", ContentType::Xml),
            ("text/html", ContentType::Html),
            ("text/plain", ContentType::Text),
            ("text/csv", ContentType::Text),
            ("application/x-www-form-urlencoded", ContentType::Form),
            (
                "multipart/form-data; boundary=ParcelBoundary",
                ContentType::Multipart,
            ),
            ("application/octet-stream", ContentType::Json),
        ];
        for (header, expected) in cases {
            assert_eq!(ContentType::from_header(header), expected, "{header}");
        }
    }
}
