//! Built in `{{$...}}` variables, such as `{{$guid}}` or `{{$timestamp}}`.
//!
//! Names follow the VS Code REST Client, the JetBrains HTTP Client and
//! Postman, so requests imported from any of them keep working. Every use
//! makes a new value.
//!
//! Arguments come either space separated, `{{$randomInt 1 7}}`, or in
//! parentheses, `{{$random.integer(1, 7)}}`. Time based variables take an
//! optional offset such as `-1 d` or `2 h`, in the units `y M w d h m s ms`.

use chrono::{DateTime, Datelike, Duration, FixedOffset, Months, Timelike, Utc};

/// Resolve the variable `expr`, the text after `$`, now. `None` when the
/// name is unknown or its arguments do not make sense.
pub fn resolve(expr: &str) -> Option<String> {
    let mut rng = fastrand::Rng::new();
    let env = |name: &str| std::env::var(name).ok();
    resolve_with(expr, chrono::Local::now().fixed_offset(), &mut rng, &env)
}

/// [`resolve`] with the clock, randomness and process environment given.
pub fn resolve_with(
    expr: &str,
    now: DateTime<FixedOffset>,
    rng: &mut fastrand::Rng,
    env: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    let (name, args) = split(expr.trim())?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let utc = now.with_timezone(&Utc);

    let value = match name {
        "guid" | "uuid" | "randomUUID" | "random.uuid" => {
            let mut bytes = [0u8; 16];
            rng.fill(&mut bytes);
            uuid::Builder::from_random_bytes(bytes)
                .into_uuid()
                .to_string()
        }
        "nanoid" | "random.nanoid" => {
            const ALPHABET: &[u8] =
                b"_-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
            let len = count(&args, 21)?;
            (0..len)
                .map(|_| ALPHABET[rng.usize(..ALPHABET.len())] as char)
                .collect()
        }
        "timestamp" => offset(utc, &args)?.timestamp().to_string(),
        "timestampMs" | "timestamp_ms" => offset(utc, &args)?.timestamp_millis().to_string(),
        "isoTimestamp" | "random.isoTimestamp" => offset(utc, &args)?
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        "datetime" => {
            let (format, rest) = args.split_first()?;
            format_datetime(offset(utc.fixed_offset(), rest)?, format, true)?
        }
        "localDatetime" => {
            let (format, rest) = args.split_first()?;
            format_datetime(offset(now, rest)?, format, false)?
        }
        "date" => offset(now, &args)?.format("%Y-%m-%d").to_string(),
        "today" => now.format("%Y-%m-%d").to_string(),
        "tomorrow" => (now + Duration::days(1)).format("%Y-%m-%d").to_string(),
        "yesterday" => (now - Duration::days(1)).format("%Y-%m-%d").to_string(),
        "randomInt" | "random.integer" => {
            let (min, max) = match args.as_slice() {
                [] => (0, 1001),
                [min, max] => (min.parse().ok()?, max.parse().ok()?),
                _ => return None,
            };
            if min >= max {
                return None;
            }
            rng.i64(min..max).to_string()
        }
        "random.float" => {
            let (min, max): (f64, f64) = match args.as_slice() {
                [] => (0.0, 1000.0),
                [min, max] => (min.parse().ok()?, max.parse().ok()?),
                _ => return None,
            };
            if min.partial_cmp(&max) != Some(std::cmp::Ordering::Less) {
                return None;
            }
            (min + rng.f64() * (max - min)).to_string()
        }
        "randomDigits" | "random.digits" => {
            let len = count(&args, 6)?;
            (0..len).map(|_| rng.digit(10)).collect()
        }
        "random.alphabetic" => {
            let len = count(&args, 10)?;
            (0..len).map(|_| rng.alphabetic()).collect()
        }
        "random.alphanumeric" => {
            let len = count(&args, 10)?;
            (0..len).map(|_| rng.alphanumeric()).collect()
        }
        "random.hexadecimal" => {
            let len = count(&args, 10)?;
            (0..len).map(|_| rng.digit(16)).collect()
        }
        "randomBoolean" | "random.bool" | "random.boolean" => rng.bool().to_string(),
        "randomZipCode" | "random.zipCode" => format!("{:05}", rng.u32(0..100_000)),
        "randomEmail" | "random.email" => format!(
            "{}.{}@example.com",
            pick(rng, FIRST_NAMES).to_lowercase(),
            pick(rng, LAST_NAMES).to_lowercase()
        ),
        "randomFirstName" | "random.firstName" => pick(rng, FIRST_NAMES).to_string(),
        "randomLastName" | "random.lastName" => pick(rng, LAST_NAMES).to_string(),
        "randomFullName" | "random.fullName" => {
            format!("{} {}", pick(rng, FIRST_NAMES), pick(rng, LAST_NAMES))
        }
        "randomCity" | "random.city" => pick(rng, CITIES).to_string(),
        "randomPhoneNumber" | "random.phoneNumber" => {
            format!("+1-555-{:03}-{:04}", rng.u32(100..1000), rng.u32(0..10_000))
        }
        "randomWord" | "random.word" => pick(rng, WORDS).to_string(),
        "processEnv" => {
            let [name] = args.as_slice() else {
                return None;
            };
            env(name.trim_start_matches('%'))?
        }
        _ => return None,
    };
    Some(value)
}

/// Split `name args...` or `name(arg, arg)` into the name and its arguments.
fn split(expr: &str) -> Option<(&str, Vec<String>)> {
    let name_end = expr
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(expr.len());
    let name = &expr[..name_end];
    let rest = expr[name_end..].trim_start();
    if name.is_empty() {
        return None;
    }

    if let Some(inner) = rest.strip_prefix('(') {
        let inner = inner.strip_suffix(')')?;
        let args = if inner.trim().is_empty() {
            Vec::new()
        } else {
            inner.split(',').map(|arg| unquote(arg.trim())).collect()
        };
        return Some((name, args));
    }

    let mut args = Vec::new();
    let mut chars = rest.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut arg = String::new();
        if c == '"' || c == '\'' {
            chars.next();
            for c2 in chars.by_ref() {
                if c2 == c {
                    break;
                }
                arg.push(c2);
            }
        } else {
            while let Some(&c2) = chars.peek() {
                if c2.is_whitespace() {
                    break;
                }
                arg.push(c2);
                chars.next();
            }
        }
        args.push(arg);
    }
    Some((name, args))
}

fn unquote(arg: &str) -> String {
    for quote in ['"', '\''] {
        if let Some(inner) = arg.strip_prefix(quote).and_then(|a| a.strip_suffix(quote)) {
            return inner.to_string();
        }
    }
    arg.to_string()
}

/// A length argument, or `default` without one. Capped so a typo cannot
/// make a huge string.
fn count(args: &[&str], default: usize) -> Option<usize> {
    let len = match args {
        [] => default,
        [len] => len.parse().ok()?,
        _ => return None,
    };
    (len <= 10_000).then_some(len)
}

/// Apply an optional `<amount> <unit>` offset to a time.
fn offset<Tz: chrono::TimeZone>(time: DateTime<Tz>, args: &[&str]) -> Option<DateTime<Tz>> {
    let (amount, unit) = match args {
        [] => return Some(time),
        [amount, unit] => (amount.parse::<i64>().ok()?, *unit),
        _ => return None,
    };
    let months = |n: i64| Months::new(n.unsigned_abs().try_into().ok()?).into();
    let shift_months = |time: DateTime<Tz>, n: i64| -> Option<DateTime<Tz>> {
        let months: Option<Months> = months(n);
        if n >= 0 {
            time.checked_add_months(months?)
        } else {
            time.checked_sub_months(months?)
        }
    };
    match unit {
        "y" => shift_months(time, amount.checked_mul(12)?),
        "M" => shift_months(time, amount),
        "w" => time.checked_add_signed(Duration::try_weeks(amount)?),
        "d" => time.checked_add_signed(Duration::try_days(amount)?),
        "h" => time.checked_add_signed(Duration::try_hours(amount)?),
        "m" => time.checked_add_signed(Duration::try_minutes(amount)?),
        "s" => time.checked_add_signed(Duration::try_seconds(amount)?),
        "ms" => time.checked_add_signed(Duration::try_milliseconds(amount)?),
        _ => None,
    }
}

/// Format for `$datetime` and `$localDatetime`: `iso8601`, `rfc1123`, or a
/// day.js style format such as `YYYY-MM-DD HH:mm`.
fn format_datetime(time: DateTime<FixedOffset>, format: &str, utc: bool) -> Option<String> {
    match format {
        "iso8601" if utc => Some(time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        "iso8601" => Some(time.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()),
        "rfc1123" => Some(
            time.with_timezone(&Utc)
                .format("%a, %d %b %Y %H:%M:%S GMT")
                .to_string(),
        ),
        custom => Some(format_custom(time, custom)),
    }
}

/// Format with day.js tokens. Text in `[brackets]` is copied as it is.
fn format_custom(time: DateTime<FixedOffset>, format: &str) -> String {
    const TOKENS: &[&str] = &[
        "YYYY", "MMMM", "dddd", "SSS", "MMM", "ddd", "YY", "MM", "DD", "HH", "hh", "mm", "ss",
        "ZZ", "M", "D", "H", "h", "m", "s", "A", "a", "Z", "X", "x",
    ];
    let mut out = String::new();
    let mut rest = format;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('[')
            && let Some(end) = after.find(']')
        {
            out.push_str(&after[..end]);
            rest = &after[end + 1..];
            continue;
        }
        let Some(token) = TOKENS.iter().find(|token| rest.starts_with(*token)) else {
            let c = rest.chars().next().unwrap_or_default();
            out.push(c);
            rest = &rest[c.len_utf8()..];
            continue;
        };
        let piece = match *token {
            "YYYY" => time.format("%Y").to_string(),
            "YY" => time.format("%y").to_string(),
            "MMMM" => time.format("%B").to_string(),
            "MMM" => time.format("%b").to_string(),
            "MM" => format!("{:02}", time.month()),
            "M" => time.month().to_string(),
            "DD" => format!("{:02}", time.day()),
            "D" => time.day().to_string(),
            "dddd" => time.format("%A").to_string(),
            "ddd" => time.format("%a").to_string(),
            "HH" => format!("{:02}", time.hour()),
            "H" => time.hour().to_string(),
            "hh" => time.format("%I").to_string(),
            "h" => time.format("%-I").to_string(),
            "mm" => format!("{:02}", time.minute()),
            "m" => time.minute().to_string(),
            "ss" => format!("{:02}", time.second()),
            "s" => time.second().to_string(),
            "SSS" => format!("{:03}", time.timestamp_subsec_millis()),
            "A" => time.format("%p").to_string(),
            "a" => time.format("%P").to_string(),
            "ZZ" => time.format("%z").to_string(),
            "Z" => time.format("%:z").to_string(),
            "X" => time.timestamp().to_string(),
            _ => time.timestamp_millis().to_string(),
        };
        out.push_str(&piece);
        rest = &rest[token.len()..];
    }
    out
}

fn pick<'a>(rng: &mut fastrand::Rng, items: &[&'a str]) -> &'a str {
    items[rng.usize(..items.len())]
}

const FIRST_NAMES: &[&str] = &[
    "Ada", "Alan", "Astrid", "Bjorn", "Chen", "Dana", "Elin", "Farah", "Grace", "Hugo", "Ines",
    "Jonas", "Kofi", "Lena", "Maya", "Nils", "Omar", "Priya", "Rosa", "Sven", "Tara", "Yuki",
];

const LAST_NAMES: &[&str] = &[
    "Andersson",
    "Berg",
    "Costa",
    "Dubois",
    "Eriksson",
    "Fischer",
    "Garcia",
    "Hansen",
    "Ito",
    "Johansson",
    "Kim",
    "Lindqvist",
    "Mendes",
    "Nakamura",
    "Okafor",
    "Patel",
    "Rossi",
    "Smith",
];

const CITIES: &[&str] = &[
    "Gothenburg",
    "Lisbon",
    "Montreal",
    "Nairobi",
    "Osaka",
    "Porto",
    "Reykjavik",
    "Seoul",
    "Stockholm",
    "Toronto",
    "Umea",
    "Valencia",
];

const WORDS: &[&str] = &[
    "anchor", "breeze", "cobalt", "delta", "ember", "falcon", "glacier", "harbor", "island",
    "juniper", "kestrel", "lantern", "meadow", "nimbus", "orbit", "parcel", "quartz", "river",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<FixedOffset> {
        // 2026-03-04 13:05:09.042 in UTC+2.
        DateTime::parse_from_rfc3339("2026-03-04T13:05:09.042+02:00").unwrap()
    }

    fn eval(expr: &str) -> Option<String> {
        let mut rng = fastrand::Rng::with_seed(7);
        let env = |name: &str| (name == "HOME").then(|| "/home/kestrel".to_string());
        resolve_with(expr, now(), &mut rng, &env)
    }

    #[test]
    fn uuids_have_the_v4_shape() {
        for name in ["guid", "uuid", "randomUUID", "random.uuid"] {
            let id = eval(name).unwrap();
            let parsed = uuid::Uuid::parse_str(&id).unwrap();
            assert_eq!(parsed.get_version_num(), 4, "{name}");
        }
    }

    #[test]
    fn timestamps_and_offsets() {
        let seconds = now().timestamp();
        assert_eq!(eval("timestamp").unwrap(), seconds.to_string());
        assert_eq!(
            eval("timestamp -1 d").unwrap(),
            (seconds - 86_400).to_string()
        );
        assert_eq!(
            eval("timestamp 2 h").unwrap(),
            (seconds + 7_200).to_string()
        );
        assert_eq!(
            eval("timestampMs").unwrap(),
            now().timestamp_millis().to_string()
        );
        assert_eq!(eval("isoTimestamp").unwrap(), "2026-03-04T11:05:09.042Z");
        assert_eq!(eval("timestamp 1 parsec"), None);
    }

    #[test]
    fn datetimes() {
        assert_eq!(
            eval("datetime iso8601").unwrap(),
            "2026-03-04T11:05:09.042Z"
        );
        assert_eq!(
            eval("datetime rfc1123").unwrap(),
            "Wed, 04 Mar 2026 11:05:09 GMT"
        );
        assert_eq!(
            eval("datetime \"YYYY-MM-DD HH:mm\"").unwrap(),
            "2026-03-04 11:05"
        );
        assert_eq!(eval("datetime 'DD/MM/YY' 1 y").unwrap(), "04/03/27");
        assert_eq!(
            eval("localDatetime iso8601").unwrap(),
            "2026-03-04T13:05:09.042+02:00"
        );
        assert_eq!(
            eval("localDatetime \"dddd D MMMM [at] h:mm A\"").unwrap(),
            "Wednesday 4 March at 1:05 PM"
        );
        assert_eq!(
            eval("localDatetime \"X x SSS Z\"")
                .unwrap()
                .split(' ')
                .count(),
            4
        );
        assert_eq!(eval("datetime"), None);
    }

    #[test]
    fn dates() {
        assert_eq!(eval("date").unwrap(), "2026-03-04");
        assert_eq!(eval("today").unwrap(), "2026-03-04");
        assert_eq!(eval("tomorrow").unwrap(), "2026-03-05");
        assert_eq!(eval("yesterday").unwrap(), "2026-03-03");
        assert_eq!(eval("date -1 w").unwrap(), "2026-02-25");
        assert_eq!(eval("date -1 M").unwrap(), "2026-02-04");
    }

    #[test]
    fn random_numbers_stay_in_range() {
        for _ in 0..50 {
            let n: i64 = eval("randomInt 3 5").unwrap().parse().unwrap();
            assert!((3..5).contains(&n));
            let n: i64 = eval("random.integer(-2, 0)").unwrap().parse().unwrap();
            assert!((-2..0).contains(&n));
            let n: i64 = eval("randomInt").unwrap().parse().unwrap();
            assert!((0..=1000).contains(&n));
            let f: f64 = eval("random.float(0.5, 1.5)").unwrap().parse().unwrap();
            assert!((0.5..1.5).contains(&f));
        }
        assert_eq!(eval("randomInt 5 5"), None);
        assert_eq!(eval("randomInt 1"), None);
    }

    #[test]
    fn random_strings_have_their_shape() {
        let id = eval("nanoid").unwrap();
        assert_eq!(id.len(), 21);
        assert_eq!(eval("nanoid 8").unwrap().len(), 8);
        assert!(
            eval("randomDigits 4")
                .unwrap()
                .bytes()
                .all(|b| b.is_ascii_digit())
        );
        assert_eq!(eval("randomDigits").unwrap().len(), 6);
        assert!(
            eval("random.alphabetic(12)")
                .unwrap()
                .chars()
                .all(|c| c.is_ascii_alphabetic())
        );
        assert!(
            eval("random.hexadecimal(9)")
                .unwrap()
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        );
        assert_eq!(eval("random.alphanumeric(3)").unwrap().len(), 3);
        let zip = eval("randomZipCode").unwrap();
        assert!(zip.len() == 5 && zip.bytes().all(|b| b.is_ascii_digit()));
        assert!(eval("randomEmail").unwrap().ends_with("@example.com"));
        assert!(eval("randomFullName").unwrap().contains(' '));
        assert!(matches!(
            eval("randomBoolean").as_deref(),
            Some("true" | "false")
        ));
        assert_eq!(eval("nanoid 100000"), None);
    }

    #[test]
    fn process_environment() {
        assert_eq!(eval("processEnv HOME").unwrap(), "/home/kestrel");
        assert_eq!(eval("processEnv %HOME").unwrap(), "/home/kestrel");
        assert_eq!(eval("processEnv NOPE"), None);
    }

    #[test]
    fn unknown_names_resolve_to_nothing() {
        assert_eq!(eval("dotenv SECRET"), None);
        assert_eq!(eval("nope"), None);
        assert_eq!(eval(""), None);
    }

    #[test]
    fn every_name_resolves() {
        const NAMES: &[&str] = &[
            "$guid",
            "$uuid",
            "$randomUUID",
            "$random.uuid",
            "$nanoid",
            "$timestamp",
            "$timestampMs",
            "$isoTimestamp",
            "$datetime",
            "$localDatetime",
            "$date",
            "$today",
            "$tomorrow",
            "$yesterday",
            "$randomInt",
            "$random.integer",
            "$random.float",
            "$randomDigits",
            "$random.alphabetic",
            "$random.alphanumeric",
            "$random.hexadecimal",
            "$randomBoolean",
            "$randomZipCode",
            "$randomEmail",
            "$randomFirstName",
            "$randomLastName",
            "$randomFullName",
            "$randomCity",
            "$randomPhoneNumber",
            "$randomWord",
            "$processEnv",
        ];
        for name in NAMES {
            let expr = match *name {
                "$datetime" | "$localDatetime" => format!("{} iso8601", &name[1..]),
                "$random.integer" | "$random.float" => format!("{}(1, 2)", &name[1..]),
                "$random.alphabetic" | "$random.alphanumeric" | "$random.hexadecimal" => {
                    format!("{}(4)", &name[1..])
                }
                "$processEnv" => "processEnv HOME".to_string(),
                _ => name[1..].to_string(),
            };
            assert!(eval(&expr).is_some(), "{name} did not resolve");
        }
    }
}
