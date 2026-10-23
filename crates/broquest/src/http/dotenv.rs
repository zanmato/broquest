//! `{{$dotenv NAME}}` in `.http` requests: a value from the `.env` file in
//! the `.http` file's folder, as in the VS Code REST Client.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::domain::RequestData;

/// The values of `dir/.env`, empty when there is none.
pub fn load(dir: &Path) -> HashMap<String, String> {
    fs::read_to_string(dir.join(".env"))
        .map(|text| parse(&text))
        .unwrap_or_default()
}

/// `KEY=value` lines, with `export`, quotes, blank lines and `#` comments.
fn parse(text: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        let value = match value.chars().next() {
            Some(quote @ ('"' | '\'')) => {
                let inner = &value[1..];
                let inner = inner.find(quote).map_or(inner, |end| &inner[..end]);
                if quote == '"' {
                    inner.replace("\\n", "\n").replace("\\\"", "\"")
                } else {
                    inner.to_string()
                }
            }
            // An unquoted value ends at a ` #` comment.
            _ => value
                .split_once(" #")
                .map_or(value, |(value, _)| value.trim_end())
                .to_string(),
        };
        values.insert(key.trim().to_string(), value);
    }
    values
}

/// Replace `{{$dotenv NAME}}` with its value. Unknown names keep their
/// placeholder.
pub fn expand(text: &str, values: &HashMap<String, String>) -> String {
    if !text.contains("$dotenv") {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let inner = after[..end].trim();
        let value = inner
            .strip_prefix("$dotenv")
            .filter(|name| name.starts_with(char::is_whitespace))
            .map(|name| name.trim().trim_start_matches('%'))
            .and_then(|name| values.get(name));
        match value {
            Some(value) => out.push_str(value),
            None => out.push_str(&rest[start..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Fill in `{{$dotenv NAME}}` across a request and its variables.
pub fn expand_request(request: &mut RequestData, values: &HashMap<String, String>) {
    if values.is_empty() {
        return;
    }
    request.url = expand(&request.url, values);
    request.body = expand(&request.body, values);
    for pair in request
        .headers
        .iter_mut()
        .chain(request.query_params.iter_mut())
        .chain(request.path_params.iter_mut())
        .chain(request.vars.iter_mut())
    {
        pair.value = expand(&pair.value, values);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dotenv_files() {
        let values = parse(
            "# comment\nexport TOKEN=abc\nEMPTY=\nQUOTED=\"a b # c\"\nSINGLE='x\\ny'\nINLINE=value # note\nNEWLINE=\"l1\\nl2\"\nnot a line\n",
        );
        assert_eq!(values["TOKEN"], "abc");
        assert_eq!(values["EMPTY"], "");
        assert_eq!(values["QUOTED"], "a b # c");
        assert_eq!(values["SINGLE"], "x\\ny");
        assert_eq!(values["INLINE"], "value");
        assert_eq!(values["NEWLINE"], "l1\nl2");
        assert_eq!(values.len(), 6);
    }

    #[test]
    fn expands_dotenv_placeholders_only() {
        let values = HashMap::from([("TOKEN".to_string(), "abc".to_string())]);
        assert_eq!(
            expand(
                "Bearer {{$dotenv TOKEN}} {{ $dotenv %TOKEN }} {{host}} {{$dotenv NOPE}}",
                &values
            ),
            "Bearer abc abc {{host}} {{$dotenv NOPE}}"
        );
        assert_eq!(
            expand("{{$dotenvTOKEN}} {{", &values),
            "{{$dotenvTOKEN}} {{"
        );
    }
}
