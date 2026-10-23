//! Editing keeps every byte it does not need to change.

use http_file::edit::{append_request, remove_block, render, set_variables, update_request};
use http_file::{NewRequest, RequestEdit, apply, parse};

const FILE: &str = "\
@host = https://api.example.com
@version = v1

### Log in
# a comment that must survive
# @name login
POST {{host}}/{{version}}/login
Content-Type: application/json
# between headers
Accept: application/json

{\"user\": \"kestrel\"}

> {% client.global.set(\"token\", response.body.token); %}

### List shipments
GET {{host}}/{{version}}/shipments
    ?status=in_transit
Authorization: Bearer {{token}}

###
GET {{host}}/{{version}}/health
";

fn edited(source: &str, block: usize, edit: &RequestEdit) -> String {
    let file = parse(source);
    apply(source, &update_request(&file, block, edit))
}

#[test]
fn an_empty_edit_changes_nothing() {
    for block in 1..4 {
        assert_eq!(edited(FILE, block, &RequestEdit::default()), FILE);
    }
    let same = RequestEdit {
        name: Some("login".into()),
        method: Some("post".into()),
        url: Some("{{host}}/{{version}}/login".into()),
        headers: Some(vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ]),
        body: Some("{\"user\": \"kestrel\"}".into()),
        response_handler: Some(Some(
            " client.global.set(\"token\", response.body.token); ".into(),
        )),
        ..Default::default()
    };
    assert_eq!(edited(FILE, 1, &same), FILE);
}

#[test]
fn changing_a_header_value_touches_only_that_value() {
    let out = edited(
        FILE,
        1,
        &RequestEdit {
            headers: Some(vec![
                ("Content-Type".into(), "application/xml".into()),
                ("Accept".into(), "application/json".into()),
            ]),
            ..Default::default()
        },
    );
    assert_eq!(
        out,
        FILE.replacen(
            "Content-Type: application/json",
            "Content-Type: application/xml",
            1
        )
    );
}

#[test]
fn adding_a_header_rewrites_the_header_lines() {
    let out = edited(
        FILE,
        2,
        &RequestEdit {
            headers: Some(vec![
                ("Authorization".into(), "Bearer {{token}}".into()),
                ("X-Trace".into(), "on".into()),
            ]),
            ..Default::default()
        },
    );
    assert_eq!(
        out,
        FILE.replace(
            "Authorization: Bearer {{token}}\n",
            "Authorization: Bearer {{token}}\nX-Trace: on\n"
        )
    );
    let request = parse(&out).requests().nth(1).cloned().unwrap();
    assert_eq!(request.headers.len(), 2);
}

#[test]
fn method_url_and_name() {
    let out = edited(
        FILE,
        2,
        &RequestEdit {
            name: Some("Shipments in transit".into()),
            method: Some("POST".into()),
            url: Some("{{host}}/v2/shipments?status=in_transit".into()),
            ..Default::default()
        },
    );
    assert!(out.contains(
        "### Shipments in transit\nPOST {{host}}/v2/shipments?status=in_transit\nAuthorization"
    ));
    assert!(out.contains("# a comment that must survive"));

    // A request without a method or name gets both added.
    let out = edited(
        "GET /a\n###\n/health\n",
        1,
        &RequestEdit {
            name: Some("health".into()),
            method: Some("HEAD".into()),
            ..Default::default()
        },
    );
    assert_eq!(out, "GET /a\n###\n# @name health\nHEAD /health\n");
}

#[test]
fn body_changes() {
    // Replace.
    let out = edited(
        FILE,
        1,
        &RequestEdit {
            body: Some("{\"user\": \"heron\"}".into()),
            ..Default::default()
        },
    );
    assert_eq!(out, FILE.replace("\"kestrel\"", "\"heron\""));

    // Remove: the blank line before it goes too, the handler stays.
    let out = edited(
        FILE,
        1,
        &RequestEdit {
            body: Some(String::new()),
            ..Default::default()
        },
    );
    assert!(out.contains("Accept: application/json\n\n> {% client.global.set"));
    assert!(!out.contains("kestrel"));

    // Add, to a request without one.
    let out = edited(
        FILE,
        3,
        &RequestEdit {
            body: Some("ping".into()),
            ..Default::default()
        },
    );
    assert!(out.ends_with("GET {{host}}/{{version}}/health\n\nping\n"));
    let added = parse(&out).requests().nth(2).cloned().unwrap();
    assert_eq!(added.body.unwrap().content.text, "ping");
}

#[test]
fn scripts_are_added_replaced_and_removed() {
    let out = edited(
        FILE,
        3,
        &RequestEdit {
            pre_request_script: Some(Some("request.variables.set(\"a\", 1);".into())),
            response_handler: Some(Some("client.log(response.status);".into())),
            ..Default::default()
        },
    );
    let request = parse(&out).requests().nth(2).cloned().unwrap();
    assert_eq!(request.pre_request_scripts.len(), 1);
    assert_eq!(request.response_handlers.len(), 1);
    assert_eq!(request.url(), "{{host}}/{{version}}/health");

    let out = edited(
        FILE,
        1,
        &RequestEdit {
            response_handler: Some(None),
            ..Default::default()
        },
    );
    assert!(!out.contains("client.global.set"));
    assert!(out.contains("{\"user\": \"kestrel\"}\n\n### List shipments"));
}

#[test]
fn crlf_files_stay_crlf() {
    let source = FILE.replace('\n', "\r\n");
    let out = edited(
        &source,
        1,
        &RequestEdit {
            body: Some("{\n  \"user\": \"heron\"\n}".into()),
            headers: Some(vec![("Content-Type".into(), "application/json".into())]),
            ..Default::default()
        },
    );
    assert!(!out.replace("\r\n", "").contains('\n'));
    assert!(out.contains("{\r\n  \"user\": \"heron\"\r\n}"));
}

#[test]
fn append_and_remove_requests() {
    let file = parse(FILE);
    let new = NewRequest {
        name: Some("Create shipment".into()),
        method: "POST".into(),
        url: "{{host}}/{{version}}/shipments".into(),
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: "{\"weight_kg\": 2}".into(),
        ..Default::default()
    };
    let out = apply(FILE, &append_request(&file, &new));
    assert_eq!(&out[..FILE.len()], FILE);
    assert_eq!(&out[FILE.len()..], format!("\n{}", render(&new, "\n")));
    let appended = parse(&out).requests().last().cloned().unwrap();
    assert_eq!(appended.method(), "POST");
    assert_eq!(appended.body.unwrap().content.text, "{\"weight_kg\": 2}");

    // Removing a block keeps the others byte for byte.
    let out = apply(FILE, &remove_block(&file, 2));
    assert!(!out.contains("List shipments"));
    assert!(out.contains("### Log in") && out.contains("/health"));

    // A first block without a separator keeps the file variables.
    let source = "@a = 1\nGET /x\n###\nGET /y\n";
    let out = apply(source, &remove_block(&parse(source), 0));
    assert_eq!(out, "@a = 1\n###\nGET /y\n");

    // Appending to a file without a trailing newline or to an empty file.
    let out = apply("GET /x", &append_request(&parse("GET /x"), &new));
    assert!(out.starts_with("GET /x\n\n### Create shipment\n"));
    let out = apply("", &append_request(&parse(""), &new));
    assert!(out.starts_with("### Create shipment\n"));
}

#[test]
fn variables_are_updated_in_place() {
    let file = parse(FILE);
    let vars = vec![
        ("host".to_string(), "http://localhost:4010".to_string()),
        ("region".to_string(), "eu".to_string()),
    ];
    let out = apply(FILE, &set_variables(&file, &vars));
    assert!(out.starts_with("@host = http://localhost:4010\n@region = eu\n\n### Log in"));

    let out = apply("GET /x\n", &set_variables(&parse("GET /x\n"), &vars));
    assert!(out.starts_with("@host = http://localhost:4010\n@region = eu\n\nGET /x"));
}

#[test]
fn edits_keep_the_corpus_lossless() {
    // Every corpus file survives an empty edit of each of its requests.
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let mut checked = 0;
    for dir in std::fs::read_dir(corpus).unwrap() {
        let dir = dir.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if !matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("http" | "rest")
            ) {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            let file = parse(&source);
            for (index, block) in file.blocks.iter().enumerate() {
                let Some(request) = &block.request else {
                    continue;
                };
                let same = RequestEdit {
                    method: Some(request.method().to_string()),
                    url: Some(request.url()),
                    headers: Some(
                        request
                            .headers
                            .iter()
                            .map(|h| (h.name.text.clone(), h.full_value()))
                            .collect(),
                    ),
                    body: Some(
                        request
                            .body
                            .as_ref()
                            .map_or(String::new(), |b| b.content.text.clone()),
                    ),
                    ..Default::default()
                };
                assert_eq!(edited(&source, index, &same), source, "{}", path.display());
                checked += 1;
            }
        }
    }
    assert!(checked > 80);
}
