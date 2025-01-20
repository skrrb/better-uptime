use serde::{Deserialize, Serialize};

/// A `Result` alias where the `Err` case is `better_uptime::Error`.
pub type Result<T> = std::result::Result<T, Error>;

const API_URL: &str = "https://uptime.betterstack.com/api";

/// The Errors that may occur while using this crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Uptime API: {0}")]
    UptimeApi(String),

    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

fn maybe_uptime_api_error<T>(value: serde_json::Value) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    #[derive(Deserialize)]
    struct ErrorResponse {
        error: String,
    }
    if let Ok(ErrorResponse { error }) = serde_json::from_value::<ErrorResponse>(value.clone()) {
        Err(Error::UptimeApi(error))
    } else {
        serde_json::from_value(value).map_err(|err| err.into())
    }
}

pub struct Uptime {
    pub token: String,
}

#[derive(Debug, Serialize, Default)]
pub struct IncidentRequest {
    pub requester_email: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_wait: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Incident {
    pub id: String,
}

impl Uptime {
    /// Ping the Uptime API to make sure that the service is up: https://betterstack.com/docs/uptime/cron-and-heartbeat-monitor/
    pub async fn heartbeat(identifier: &str) -> Result<()> {
        let url = format!("{API_URL}/v1/heartbeat/{identifier}");
        match reqwest::get(url).await?.error_for_status() {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Create a new incident: https://betterstack.com/docs/uptime/api/create-a-new-incident/
    pub async fn create_incident(&self, request: IncidentRequest) -> Result<Incident> {
        let url = format!("{API_URL}/v2/incidents");

        #[derive(Debug, Deserialize)]
        struct IncidentResponse {
            data: Incident,
        }

        let response = maybe_uptime_api_error::<IncidentResponse>(
            reqwest::Client::builder()
                .build()?
                .post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.token))
                .json(&request)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?,
        )?;

        Ok(response.data)
    }
}
