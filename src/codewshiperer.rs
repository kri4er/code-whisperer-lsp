use std::collections::HashMap;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};

const CODEWHISPERER_ENDPOINT: &str = "https://codewhisperer.us-east-1.amazonaws.com/";
const CODEWHISPERER_IDPOOL_ID: &str = "us-east-1:70717e99-906f-4add-908c-bd9074a2f5b9";
//How many caracters to the left/right of current fil position to sent
const CHARACTERS_LIMIT: i32 = 10240;

// Define structs based on the JSON schema
#[derive(Debug, Serialize, Deserialize)]
struct AccessDeniedException {
    message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum ArtifactType {
    SourceCode,
    BuiltJars,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgrammingLanguage {
    language_name: String,
}

pub struct CodeWhispererClient {
    client: Client,
    base_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCodeScanRequest {
    artivacts: HashMap<ArtifactType, String>,
    programming_language: ProgrammingLanguage,
    client_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRecommendationsRequest {
    file_context: HashMap<ArtifactType, String>,
    max_results: Option<i32>,
    next_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recommendation {
    content: String,
    references: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRecommendationsResponse {
    recommendations: Vec<Recommendation>,
    max_results: Option<i32>,
    next_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCodeScanResponse {
    job_id: String,
    status: String,
    error_message: String,
}

impl CodeWhispererClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn create_code_scan(
        &self,
        request: CreateCodeScanRequest,
    ) -> Result<CreateCodeScanResponse, reqwest::Error> {
        let url = format!("{}/CreateCodeScan", self.base_url);
        self.client
            .post(&url)
            .header(
                HeaderName::from_static("x-amzn-codewhisperer-token"),
                HeaderValue::from_static("token_value"),
            )
            .header(
                HeaderName::from_static("x-amzn-codewhisperer-optout"),
                HeaderValue::from_static("optout_value"),
            )
            .json(&request)
            .send()
            .await?
            .json()
            .await
    }
    pub async fn list_recommendations(
        &self,
        request: CreateCodeScanRequest,
    ) -> Result<CreateCodeScanResponse, reqwest::Error> {
        let url = format!("{}/CreateCodeScan", self.base_url);
        self.client
            .post(&url)
            .header(
                HeaderName::from_static("x-amzn-codewhisperer-token"),
                HeaderValue::from_static("token_value"),
            )
            .header(
                HeaderName::from_static("x-amzn-codewhisperer-optout"),
                HeaderValue::from_static("optout_value"),
            )
            .json(&request)
            .send()
            .await?
            .json()
            .await
    }
}
