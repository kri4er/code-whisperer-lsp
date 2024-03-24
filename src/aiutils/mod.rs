use std::{
    fmt,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

// basically a cursor position with some context left and right from cursor
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgrammingLanguage {
    language_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContext {
    left_file_content: String,
    right_file_content: String,
    filename: String,
    programming_language: ProgrammingLanguage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationContext {
    file_context: FileContext,
    max_results: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    content: String,
    references: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsResult {
    recommendations: Vec<Recommendation>,
}

pub trait RecommendationProvider {
    fn recomendations(&self, recomendation_context: RecommendationContext) -> Vec<Recommendation>;
}

pub struct AWSCodeWhispererCliProvider {}

impl RecommendationProvider for AWSCodeWhispererCliProvider {
    fn recomendations(&self, recomendation_context: RecommendationContext) -> Vec<Recommendation> {
        let json = serde_json::to_string(&recomendation_context)
            .expect("Can't serrialize recomendation_context");

        let output = Command::new("/usr/local/bin/aws")
            .arg("codewhisperer")
            .arg("generate-recommendations")
            .arg("--cli-input-json")
            .arg(json)
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute the process");

        let stdout = String::from_utf8(output.stdout).unwrap();
        serde_json::from_str::<RecommendationsResult>(&stdout)
            .unwrap_or_default()
            .recommendations
    }
}

#[cfg(test)]
mod test_aiutils {
    use crate::aiutils::{
        AWSCodeWhispererCliProvider, FileContext, ProgrammingLanguage, RecommendationContext,
        RecommendationProvider,
    };

    #[ignore]//Ignored since it is actually does a network call
    #[tokio::test]
    async fn test_generate_reccomendations() {
        let cli_rec_provider = AWSCodeWhispererCliProvider {};
        let context = RecommendationContext {
            file_context: FileContext {
                filename: "main.rs".into(),
                programming_language: ProgrammingLanguage {
                    language_name: "rust".into(),
                },
                left_file_content: "fn main() { let n = 10;".into(),
                right_file_content: "}".into(),
            },
            max_results: 1,
        };
        let res = cli_rec_provider.recomendations(context);

        println!("LOGME: results are: {:?}", res);
    }
}
