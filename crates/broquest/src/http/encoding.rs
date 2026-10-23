//! Decoding of compressed response bodies and their text.
//!
//! reqwest's own decompression is off, so the body arrives as sent. That
//! keeps its compressed size and the `Content-Encoding` header, which reqwest
//! would otherwise drop.

use std::io::Read;

/// Sent as `Accept-Encoding` unless the request sets its own.
pub const ACCEPT_ENCODING: &str = "gzip, deflate, br, zstd";

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    Unsupported(String),
    Corrupt { encoding: String, message: String },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(encoding) => write!(f, "unsupported content encoding {encoding}"),
            Self::Corrupt { encoding, message } => {
                write!(f, "could not decode {encoding} body: {message}")
            }
        }
    }
}

/// Whether a `Content-Encoding` value means the body is compressed.
pub fn is_encoded(content_encoding: &str) -> bool {
    codings(content_encoding).next().is_some()
}

fn codings(content_encoding: &str) -> impl DoubleEndedIterator<Item = String> + '_ {
    content_encoding
        .split(',')
        .map(|coding| coding.trim().to_ascii_lowercase())
        .filter(|coding| !coding.is_empty() && coding != "identity")
}

/// Decode `body` as described by its `Content-Encoding` header. Codings are
/// listed in the order they were applied, so they are undone in reverse.
pub fn decode(body: &[u8], content_encoding: &str) -> Result<Vec<u8>, DecodeError> {
    let mut data = body.to_vec();
    for coding in codings(content_encoding).rev() {
        data = decode_one(&data, &coding)?;
    }
    Ok(data)
}

fn decode_one(body: &[u8], coding: &str) -> Result<Vec<u8>, DecodeError> {
    let corrupt = |error: std::io::Error| DecodeError::Corrupt {
        encoding: coding.to_string(),
        message: error.to_string(),
    };
    let mut out = Vec::new();
    match coding {
        "gzip" | "x-gzip" => {
            flate2::read::MultiGzDecoder::new(body)
                .read_to_end(&mut out)
                .map_err(corrupt)?;
        }
        "deflate" => {
            // HTTP deflate is zlib wrapped, but some servers send raw deflate.
            if flate2::read::ZlibDecoder::new(body)
                .read_to_end(&mut out)
                .is_err()
            {
                out.clear();
                flate2::read::DeflateDecoder::new(body)
                    .read_to_end(&mut out)
                    .map_err(corrupt)?;
            }
        }
        "br" => {
            brotli_decompressor::Decompressor::new(body, 4096)
                .read_to_end(&mut out)
                .map_err(corrupt)?;
        }
        "zstd" => {
            let mut source = body;
            ruzstd::decoding::StreamingDecoder::new(&mut source)
                .map_err(|error| DecodeError::Corrupt {
                    encoding: coding.to_string(),
                    message: error.to_string(),
                })?
                .read_to_end(&mut out)
                .map_err(corrupt)?;
        }
        other => return Err(DecodeError::Unsupported(other.to_string())),
    }
    Ok(out)
}

/// Decode a body as text, using the charset its `Content-Type` names and
/// UTF-8 otherwise. A byte order mark wins over both, as in browsers.
pub fn decode_text(body: &[u8], content_type: &str) -> String {
    let encoding = content_type
        .split(';')
        .skip(1)
        .filter_map(|param| param.split_once('='))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("charset"))
        .and_then(|(_, value)| {
            encoding_rs::Encoding::for_label(value.trim().trim_matches('"').as_bytes())
        })
        .unwrap_or(encoding_rs::UTF_8);
    encoding.decode(body).0.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEXT: &[u8] = b"{\"shipments\": [\"KS-48213\", \"KS-48197\"]}";

    fn gzip(data: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn zlib(data: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn raw_deflate(data: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn brotli(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut writer = brotli::CompressorWriter::new(&mut out, 4096, 5, 22);
        writer.write_all(data).unwrap();
        drop(writer);
        out
    }

    fn zstd(data: &[u8]) -> Vec<u8> {
        ruzstd::encoding::compress_to_vec(data, ruzstd::encoding::CompressionLevel::Fastest)
    }

    #[test]
    fn decodes_each_supported_encoding() {
        assert_eq!(decode(&gzip(TEXT), "gzip").unwrap(), TEXT);
        assert_eq!(decode(&gzip(TEXT), "x-gzip").unwrap(), TEXT);
        assert_eq!(decode(&zlib(TEXT), "deflate").unwrap(), TEXT);
        assert_eq!(decode(&raw_deflate(TEXT), "deflate").unwrap(), TEXT);
        assert_eq!(decode(&brotli(TEXT), "br").unwrap(), TEXT);
        assert_eq!(decode(&zstd(TEXT), "zstd").unwrap(), TEXT);
    }

    #[test]
    fn undoes_stacked_encodings_in_reverse() {
        assert_eq!(decode(&brotli(&gzip(TEXT)), "gzip, br").unwrap(), TEXT);
        assert_eq!(decode(&gzip(TEXT), " GZIP , identity ").unwrap(), TEXT);
    }

    #[test]
    fn identity_and_empty_leave_the_body_alone() {
        assert_eq!(decode(TEXT, "identity").unwrap(), TEXT);
        assert_eq!(decode(TEXT, "").unwrap(), TEXT);
        assert!(!is_encoded("identity"));
        assert!(!is_encoded(""));
        assert!(is_encoded("br"));
    }

    #[test]
    fn reports_unknown_and_corrupt_bodies() {
        assert_eq!(
            decode(TEXT, "compress"),
            Err(DecodeError::Unsupported("compress".into()))
        );
        assert!(matches!(
            decode(TEXT, "gzip"),
            Err(DecodeError::Corrupt { .. })
        ));
    }

    #[test]
    fn decodes_text_in_its_charset() {
        assert_eq!(
            decode_text(b"caf\xe9", "text/plain; charset=ISO-8859-1"),
            "café"
        );
        assert_eq!(
            decode_text(b"caf\xe9", "text/plain; Charset=\"latin1\""),
            "café"
        );
        assert_eq!(decode_text("café".as_bytes(), "application/json"), "café");
        assert_eq!(decode_text("café".as_bytes(), ""), "café");
    }
}
