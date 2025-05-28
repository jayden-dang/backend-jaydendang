use std::sync::LazyLock;

use regex::Regex;

pub static USERNAME_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

pub static TIMEZONE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[A-Z][a-z]+/[A-Z][a-z_]+$").unwrap());

pub static COUNTRY_CODE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[A-Z]{2}$").unwrap());
