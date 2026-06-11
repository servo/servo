/// JS use aconym perserving style, like `browsingContext.SetBypassCSPResult`, while
/// we say `SetBypassCspResult` in Rust.
pub fn normalize_acronym(iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    let mut iter = iter.peekable();
    let mut last_upper = false;

    std::iter::from_fn(move || {
        let c = iter.next()?;
        let cur_upper = c.is_ascii_uppercase();
        let next_upper = iter.peek().map(char::is_ascii_uppercase).unwrap_or(true);

        let out = match last_upper && cur_upper && next_upper {
            true => c.to_ascii_lowercase(),
            false => c,
        };

        last_upper = cur_upper;
        Some(out)
    })
}

/// Convert PascalCase or camelCase to snake_case.
///
/// this does not handle snake_case.
pub fn to_snake_case(iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    let mut iter = normalize_acronym(iter).peekable();
    let mut tmp = None;

    std::iter::from_fn(move || {
        if let Some(tmp) = tmp.take() {
            return Some(tmp);
        }

        let cur = iter.next()?.to_ascii_lowercase();
        if iter.peek().is_some_and(char::is_ascii_uppercase) {
            tmp = Some('_');
        };

        Some(cur)
    })
}

/// Convert kebab-case, camelCase and "space case" to PascalCase.
pub fn to_pascal_case(iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    let mut iter = normalize_acronym(iter);
    let mut next_upper = true;

    std::iter::from_fn(move || {
        loop {
            let cur = iter.next()?;

            if matches!(cur, '-' | '_' | ' ') {
                next_upper = true;
                continue;
            }

            let out = match next_upper {
                true => cur.to_ascii_uppercase(),
                false => cur,
            };

            next_upper = false;
            return Some(out);
        }
    })
}

pub fn is_rust_keyword(s: &str) -> bool {
    // incomplete
    matches!(s, "type")
}

#[cfg(test)]
mod tests {
    use crate::util::{normalize_acronym, to_pascal_case, to_snake_case};

    #[test]
    fn test_normalize_acronym() {
        let input = "SetBypassCSPResult";
        let expected = "SetBypassCspResult";
        let actual: String = normalize_acronym(input.chars()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_snake_case() {
        let input = "SetBypassCSPResult";
        let expected = "set_bypass_csp_result";
        let actual: String = to_snake_case(input.chars()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_pascal_case() {
        let input = "set_bypass-CSPResult_js-uint";
        let expected = "SetBypassCspResultJsUint";
        let actual: String = to_pascal_case(input.chars()).collect();
        assert_eq!(actual, expected);
    }
}
