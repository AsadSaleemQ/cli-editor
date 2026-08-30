pub(crate) fn normalized_version(value: &str) -> &str {
    value
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric()
                    && character != '.'
                    && character != '-'
                    && character != '+'
            })
        })
        .find(|token| semver::Version::parse(token).is_ok())
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn extracts_codex_semver_tokens() {
        assert_eq!(super::normalized_version("codex-cli 0.148.0"), "0.148.0");
        assert_eq!(super::normalized_version("unknown"), "unknown");
    }
}
