use anyhow::Result;
use serde::Serialize;

use crate::classifier::RulesClassifier;
use crate::cli::OutputFormat;
use crate::scanner::ScanResult;

use super::{ConfigFinding, Finding};

#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub files_scanned: usize,
    pub total_findings: usize,
    pub total_configs: usize,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub configs: Vec<ConfigFinding>,
}

pub struct OutputFormatter;

impl OutputFormatter {
    pub fn format(
        results: &[ScanResult],
        classifier: &RulesClassifier,
        format: OutputFormat,
    ) -> Result<String> {
        let output = Self::build_output(results, classifier);

        match format {
            OutputFormat::Json => Ok(serde_json::to_string_pretty(&output)?),
            OutputFormat::Cbom => {
                tracing::warn!("CBOM output not yet implemented, using JSON");
                Ok(serde_json::to_string_pretty(&output)?)
            }
        }
    }

    pub fn build_output(results: &[ScanResult], classifier: &RulesClassifier) -> JsonOutput {
        let findings: Vec<Finding> = results
            .iter()
            .flat_map(|r| {
                r.calls
                    .iter()
                    .map(|call| Finding::from_crypto_call(call, classifier))
            })
            .collect();

        let configs: Vec<ConfigFinding> = results
            .iter()
            .flat_map(|r| r.configs.iter().map(ConfigFinding::from_crypto_config))
            .collect();

        let total_findings = findings.len();
        let total_configs = configs.len();

        JsonOutput {
            files_scanned: results.len(),
            total_findings,
            total_configs,
            findings,
            configs,
        }
    }
}
