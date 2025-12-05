#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnresolvedSource {
    FunctionParameter,
    ConfigValue,
    RuntimeValue,
    ExternalDependency,
    IdentifierNotFound,
    CycleDetected,
    NotImplemented,
    PartiallyResolved,
    MixedResolution,
    MixedTypes,
    Unknown,
}

impl UnresolvedSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FunctionParameter => "function_parameter",
            Self::ConfigValue => "config_value",
            Self::RuntimeValue => "runtime_value",
            Self::ExternalDependency => "external_dependency",
            Self::IdentifierNotFound => "identifier_not_found",
            Self::CycleDetected => "cycle_detected",
            Self::NotImplemented => "not_implemented",
            Self::PartiallyResolved => "partially_resolved",
            Self::MixedResolution => "mixed_resolution",
            Self::MixedTypes => "mixed_types",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for UnresolvedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<UnresolvedSource> for String {
    fn from(source: UnresolvedSource) -> Self {
        source.as_str().to_string()
    }
}

pub const NOT_RESOLVED: &str = "not_resolved";
pub const UNRESOLVED: &str = "unresolved";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_as_str() {
        assert_eq!(
            UnresolvedSource::FunctionParameter.as_str(),
            "function_parameter"
        );
        assert_eq!(UnresolvedSource::CycleDetected.as_str(), "cycle_detected");
    }

    #[test]
    fn test_source_display() {
        assert_eq!(
            format!("{}", UnresolvedSource::NotImplemented),
            "not_implemented"
        );
    }
}
