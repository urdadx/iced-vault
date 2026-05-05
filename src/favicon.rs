use directories::ProjectDirs;
use url::Url;

const TWO_PART_PUBLIC_SUFFIXES: [&str; 18] = [
    "co.uk", "org.uk", "ac.uk", "gov.uk", "co.jp", "ne.jp", "or.jp", "com.au", "net.au", "org.au",
    "com.br", "com.mx", "co.nz", "com.sg", "com.tr", "com.cn", "com.hk", "co.za",
];

pub(crate) fn cached_favicon_path(website_url: &str) -> Option<String> {
    let apex_domain = apex_domain(website_url)?;
    let cache_path = favicon_cache_dir()?.join(format!("{}.png", cache_file_name(&apex_domain)));

    if cache_path.exists() {
        return Some(cache_path.to_string_lossy().into_owned());
    }

    None
}

pub(crate) fn apex_domain(website_url: &str) -> Option<String> {
    let host = parse_host(website_url)?;
    let labels: Vec<&str> = host.split('.').filter(|label| !label.is_empty()).collect();

    if labels.len() <= 2 {
        return Some(host);
    }

    let suffix = labels[labels.len() - 2..].join(".");
    let label_count = if TWO_PART_PUBLIC_SUFFIXES.contains(&suffix.as_str()) {
        3
    } else {
        2
    };

    Some(labels[labels.len() - label_count..].join("."))
}

fn parse_host(website_url: &str) -> Option<String> {
    let trimmed = website_url.trim();

    if trimmed.is_empty() {
        return None;
    }

    let parsed = Url::parse(trimmed)
        .or_else(|_| Url::parse(&format!("https://{trimmed}")))
        .ok()?;
    let host = parsed
        .host_str()?
        .trim_start_matches("www.")
        .to_ascii_lowercase();

    (!host.is_empty()).then_some(host)
}

fn favicon_cache_dir() -> Option<std::path::PathBuf> {
    let path = ProjectDirs::from("dev", "iced-vault", "iced-vault")?
        .cache_dir()
        .join("favicons");

    std::fs::create_dir_all(&path).ok()?;

    Some(path)
}

fn cache_file_name(domain: &str) -> String {
    domain
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '.' || *character == '-'
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::apex_domain;

    #[test]
    fn extracts_basic_apex_domains() {
        assert_eq!(
            apex_domain("https://accounts.google.com/login").as_deref(),
            Some("google.com")
        );
        assert_eq!(
            apex_domain("www.github.com/settings").as_deref(),
            Some("github.com")
        );
    }

    #[test]
    fn handles_common_two_part_public_suffixes() {
        assert_eq!(
            apex_domain("https://login.example.co.uk").as_deref(),
            Some("example.co.uk")
        );
    }
}
