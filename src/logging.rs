use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
    Debug,
    Trace,
}

impl Verbosity {
    pub fn from_flags(verbose: u8, quiet: bool) -> Self {
        if quiet {
            return Self::Quiet;
        }
        match verbose {
            0 => Self::Normal,
            1 => Self::Verbose,
            2 => Self::Debug,
            _ => Self::Trace,
        }
    }

    fn to_level(self) -> Level {
        match self {
            Self::Quiet => Level::ERROR,
            Self::Normal => Level::WARN,
            Self::Verbose => Level::INFO,
            Self::Debug => Level::DEBUG,
            Self::Trace => Level::TRACE,
        }
    }

    fn to_filter(self) -> String {
        let level = self.to_level();
        format!("argflow={level}")
    }
}

pub fn init(verbosity: Verbosity) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(verbosity.to_filter()));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(verbosity >= Verbosity::Debug)
        .with_line_number(verbosity >= Verbosity::Debug)
        .compact();

    match verbosity {
        Verbosity::Quiet => {
            subscriber.with_writer(std::io::sink).init();
        }
        Verbosity::Normal => {
            subscriber.without_time().init();
        }
        _ => {
            subscriber.init();
        }
    }
}

impl PartialOrd for Verbosity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Verbosity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            Self::Quiet => 0,
            Self::Normal => 1,
            Self::Verbose => 2,
            Self::Debug => 3,
            Self::Trace => 4,
        };
        let other_val = match other {
            Self::Quiet => 0,
            Self::Normal => 1,
            Self::Verbose => 2,
            Self::Debug => 3,
            Self::Trace => 4,
        };
        self_val.cmp(&other_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_from_flags() {
        assert_eq!(Verbosity::from_flags(0, true), Verbosity::Quiet);
        assert_eq!(Verbosity::from_flags(0, false), Verbosity::Normal);
        assert_eq!(Verbosity::from_flags(1, false), Verbosity::Verbose);
        assert_eq!(Verbosity::from_flags(2, false), Verbosity::Debug);
        assert_eq!(Verbosity::from_flags(3, false), Verbosity::Trace);
        assert_eq!(Verbosity::from_flags(10, false), Verbosity::Trace);
    }

    #[test]
    fn test_quiet_overrides_verbose() {
        assert_eq!(Verbosity::from_flags(3, true), Verbosity::Quiet);
    }

    #[test]
    fn test_verbosity_ordering() {
        assert!(Verbosity::Quiet < Verbosity::Normal);
        assert!(Verbosity::Normal < Verbosity::Verbose);
        assert!(Verbosity::Verbose < Verbosity::Debug);
        assert!(Verbosity::Debug < Verbosity::Trace);
    }
}
