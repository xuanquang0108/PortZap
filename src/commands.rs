#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Port(u16),
    Name(String),
    Hang,
}

pub fn parse(input: &str) -> Option<Command> {
    let t = input.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    if lower == "hang" || lower == "frozen" {
        return Some(Command::Hang);
    }
    if let Some(rest) = t.strip_prefix(':') {
        let p: u16 = rest.trim().parse().ok()?;
        if p == 0 {
            return None;
        }
        return Some(Command::Port(p));
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        let p: u16 = t.parse().ok()?;
        if p == 0 {
            return None;
        }
        return Some(Command::Port(p));
    }
    if t.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')) {
        return Some(Command::Name(lower));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_command() {
        assert_eq!(parse(":3000"), Some(Command::Port(3000)));
        assert_eq!(parse(": 8080 "), Some(Command::Port(8080)));
        assert_eq!(parse(":65535"), Some(Command::Port(65535)));
        assert_eq!(parse("3000"), Some(Command::Port(3000)));
        assert_eq!(parse(" 80 "), Some(Command::Port(80)));
    }

    #[test]
    fn invalid_port() {
        assert_eq!(parse(":0"), None);
        assert_eq!(parse(":99999"), None);
        assert_eq!(parse(":"), None);
        assert_eq!(parse(":abc"), None);
        assert_eq!(parse("0"), None);
        assert_eq!(parse("99999"), None);
    }

    #[test]
    fn name_command() {
        assert_eq!(parse("node"), Some(Command::Name("node".into())));
        assert_eq!(parse("Node"), Some(Command::Name("node".into())));
        assert_eq!(parse("chrome"), Some(Command::Name("chrome".into())));
        assert_eq!(parse("java.exe"), Some(Command::Name("java.exe".into())));
        assert_eq!(parse("my-app_1"), Some(Command::Name("my-app_1".into())));
    }

    #[test]
    fn hang_command() {
        assert_eq!(parse("hang"), Some(Command::Hang));
        assert_eq!(parse("Hang"), Some(Command::Hang));
        assert_eq!(parse("frozen"), Some(Command::Hang));
        assert_eq!(parse("FROZEN"), Some(Command::Hang));
    }

    #[test]
    fn empty_and_garbage() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("   "), None);
        assert_eq!(parse("hello world"), None);
        assert_eq!(parse("тест"), None);
    }
}
