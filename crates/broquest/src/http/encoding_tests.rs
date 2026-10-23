//! End to end checks of compressed responses against a mock server.

use std::io::Write;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::domain::{EncodedBody, HttpMethod, KeyValuePair, RequestData};
use crate::http::HttpClientService;
use crate::http::encoding::ACCEPT_ENCODING;

const BODY: &str = r#"{"id":"KS-48213","status":"in_transit","carrier":"postnord"}"#;

fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
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

fn get(url: String) -> RequestData {
    RequestData {
        method: HttpMethod::Get,
        url,
        ..Default::default()
    }
}

async fn serve(body: Vec<u8>, content_encoding: Option<&str>) -> MockServer {
    let server = MockServer::start().await;
    let mut response = ResponseTemplate::new(200)
        .insert_header("Content-Type", "application/json")
        .set_body_bytes(body);
    if let Some(content_encoding) = content_encoding {
        response = response.insert_header("Content-Encoding", content_encoding);
    }
    Mock::given(method("GET"))
        .and(path("/shipment"))
        .respond_with(response)
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn decodes_compressed_responses_and_keeps_their_size() {
    for (encoding, compressed) in [
        ("gzip", gzip(BODY.as_bytes())),
        ("br", brotli(BODY.as_bytes())),
    ] {
        let server = serve(compressed.clone(), Some(encoding)).await;
        let client = HttpClientService::new(30).unwrap();

        let (response, _) = client
            .send_request(
                get(format!("{}/shipment", server.uri())),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_or_else(|e| panic!("{encoding}: {}", e.details));

        assert_eq!(response.status_code, Some(200), "{encoding}");
        let received = server.received_requests().await.unwrap();
        assert_eq!(
            received[0].headers.get("accept-encoding").unwrap(),
            ACCEPT_ENCODING,
            "{encoding}"
        );
        assert_eq!(response.body, BODY, "{encoding}");
        assert_eq!(response.size, Some(BODY.len()), "{encoding}");
        assert_eq!(
            response.encoded,
            Some(EncodedBody {
                encoding: encoding.to_string(),
                size: compressed.len(),
            }),
            "{encoding}"
        );
        assert!(
            response
                .headers
                .iter()
                .any(|h| h.key.eq_ignore_ascii_case("content-encoding") && h.value == encoding),
            "{encoding}: Content-Encoding should stay in the response headers"
        );
        assert!(
            response
                .request_headers
                .iter()
                .any(|h| h.key == "Accept-Encoding" && h.value == ACCEPT_ENCODING),
            "{encoding}: the sent Accept-Encoding should be listed"
        );
    }
}

#[tokio::test]
async fn plain_responses_have_no_encoded_size() {
    let server = serve(BODY.as_bytes().to_vec(), None).await;
    let client = HttpClientService::new(30).unwrap();

    let (response, _) = client
        .send_request(
            get(format!("{}/shipment", server.uri())),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.body, BODY);
    assert_eq!(response.size, Some(BODY.len()));
    assert_eq!(response.encoded, None);
}

#[tokio::test]
async fn a_request_can_choose_its_own_accept_encoding() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/shipment"))
        .and(header("accept-encoding", "identity"))
        .respond_with(ResponseTemplate::new(200).set_body_string(BODY))
        .mount(&server)
        .await;
    let client = HttpClientService::new(30).unwrap();

    let mut request = get(format!("{}/shipment", server.uri()));
    request.headers.push(KeyValuePair {
        key: "accept-encoding".to_string(),
        value: "identity".to_string(),
        enabled: true,
    });
    let (response, _) = client
        .send_request(request, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(response.status_code, Some(200));
    let accept_encodings = response
        .request_headers
        .iter()
        .filter(|h| h.key.eq_ignore_ascii_case("accept-encoding"))
        .count();
    assert_eq!(accept_encodings, 1);
}

fn urlencoded_post(url: String, body: &str) -> RequestData {
    RequestData {
        method: HttpMethod::Post,
        url,
        headers: vec![KeyValuePair {
            key: "Content-Type".to_string(),
            value: "application/x-www-form-urlencoded".to_string(),
            enabled: true,
        }],
        body: body.to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn urlencoded_bodies_with_an_at_sign_stay_urlencoded() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signup"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(wiremock::matchers::body_string(
            "email=a@b.com&name=kestrel",
        ))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let client = HttpClientService::new(30).unwrap();

    let request = urlencoded_post(
        format!("{}/signup", server.uri()),
        "email=a@b.com&name=kestrel",
    );
    let (response, _) = client
        .send_request(request, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(response.status_code, Some(200));
}

#[tokio::test]
async fn urlencoded_file_fields_still_upload_as_multipart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(wiremock::matchers::body_string_contains("label-contents"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let file = std::env::temp_dir().join(format!("broquest-upload-{}.txt", std::process::id()));
    std::fs::write(&file, "label-contents").unwrap();
    let client = HttpClientService::new(30).unwrap();

    let request = urlencoded_post(
        format!("{}/upload", server.uri()),
        &format!("label=@{}&note=hi", file.display()),
    );
    let (response, _) = client
        .send_request(request, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(response.status_code, Some(200));
    let received = server.received_requests().await.unwrap();
    let content_type = received[0].headers.get("content-type").unwrap();
    assert!(
        content_type
            .to_str()
            .unwrap()
            .starts_with("multipart/form-data")
    );
    std::fs::remove_file(file).ok();
}
