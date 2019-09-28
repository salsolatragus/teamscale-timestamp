use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Read;

use chrono::DateTime;
use reqwest::header::AUTHORIZATION;
use reqwest::{Response, Url};
use serde::Deserialize;

use crate::app::App;
use crate::logger::Logger;

/// Struct for retrieving info from a TFVC repo.
pub struct Tfs<'a> {
    app: &'a App,
}

#[derive(Deserialize)]
struct ChangesetResponse {
    #[serde(rename = "createdDate")]
    created_date: String,
}

type TfsResult<T> = std::result::Result<T, TfsError>;

impl<'a> Tfs<'a> {
    pub fn new(app: &'a App) -> Tfs<'a> {
        return Tfs { app };
    }

    pub fn timestamp(&self) -> Option<String> {
        let teamproject = self.app.env_variable("SYSTEM_TEAMPROJECTID")?;
        let changeset = self.app.env_variable("BUILD_SOURCEVERSION")?;
        let collection_uri = self
            .app
            .env_variable("SYSTEM_TEAMFOUNDATIONCOLLECTIONURI")?;
        return match self.timestamp_or_error(teamproject, changeset, collection_uri) {
            Ok(timestamp) => Some(timestamp),
            Err(error) => {
                self.app.log(&format!("{}", error));
                None
            }
        };
    }

    fn timestamp_or_error(
        &self,
        teamproject: String,
        changeset: String,
        collection_uri: String,
    ) -> TfsResult<String> {
        let access_token = self.get_access_token()?;
        let url = self.create_changeset_url(collection_uri, teamproject, changeset);
        let response = self.request(url, access_token)?;
        let changeset_response = self.parse_response(response)?;
        return Tfs::parse_date(changeset_response.created_date);
    }

    fn parse_date(date_string: String) -> TfsResult<String> {
        return DateTime::parse_from_rfc3339(&date_string)
            .map(|date| format!("{}000", date.timestamp()))
            .map_err(|error| TfsError::InvalidDate(error, date_string));
    }

    fn parse_response(&self, mut response: Response) -> TfsResult<ChangesetResponse> {
        let mut string = String::new();
        response
            .read_to_string(&mut string)
            .map_err(TfsError::CannotReadRequest)?;
        return serde_json::from_str::<ChangesetResponse>(&string)
            .map_err(|error| TfsError::JsonParseFailed(error, string));
    }

    fn request(&self, url: Url, access_token: String) -> TfsResult<Response> {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap();
        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()?;
        Ok(response)
    }

    fn create_changeset_url(
        &self,
        collection_uri: String,
        teamproject: String,
        changeset: String,
    ) -> Url {
        let url_string = &format!(
            "{}/{}/_apis/tfvc/changesets/{}",
            collection_uri, teamproject, changeset
        );
        return Url::parse(url_string).unwrap();
    }

    fn get_access_token(&self) -> TfsResult<String> {
        return self
            .app
            .env_variable("SYSTEM_ACCESSTOKEN")
            .ok_or(TfsError::AccessTokenNotProvided());
    }
}

#[derive(Debug)]
enum TfsError {
    JsonParseFailed(serde_json::error::Error, String),
    RequestTimedOut(reqwest::Error),
    CannotReadRequest(std::io::Error),
    TfsServerError(reqwest::Error),
    AccessTokenNotProvided(),
    InvalidAccessToken(),
    OtherRequestError(reqwest::Error),
    InvalidDate(chrono::format::ParseError, String),
}

impl Display for TfsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TfsError::JsonParseFailed(cause, json) => write!(
                f,
                "Failed to parse JSON response from TFS: {}. Original response: {}",
                cause, json
            ),
            TfsError::RequestTimedOut(cause) => {
                write!(f, "Request to {} timed out: {}", cause.safe_url(), cause)
            }
            TfsError::TfsServerError(cause) => write!(
                f,
                "TFS returned error for request to {}: {}",
                cause.safe_url(),
                cause
            ),
            TfsError::OtherRequestError(cause) => write!(
                f,
                "Request to {} failed with HTTP status code {}: {}",
                cause.safe_url(),
                cause.safe_status(),
                cause
            ),
            TfsError::AccessTokenNotProvided() => write!(
                f,
                "Environment variable SYSTEM_ACCESSTOKEN not set. Please make sure \
                 you activated 'Additional options > Allow scripts to access OAuth token' for \
                 your pipeline job! Otherwise, the timestamp for a TFVC changeset cannot be \
                 determined."
            ),
            TfsError::InvalidAccessToken() => write!(
                f,
                "The access token provided via the environment variable \
                 SYSTEM_ACCESSTOKEN was not accepted by the TFS. Please make sure you activated \
                 'Additional options > Allow scripts to access OAuth token' for your pipeline \
                 job, which will set the correct SYSTEM_ACCESSTOKEN environmen tvariable! \
                 Otherwise, the timestamp for a TFVC changeset cannot be determined"
            ),
            TfsError::InvalidDate(cause, date_string) => {
                write!(f, "TFS returned unparsable date {}: {}", date_string, cause)
            }
            TfsError::CannotReadRequest(cause) => {
                write!(f, "Failed to read request body: {}", cause)
            }
        }
    }
}

trait SafeRequestErrorProps {
    fn safe_url(&self) -> String;
    fn safe_status(&self) -> String;
}

impl SafeRequestErrorProps for reqwest::Error {
    fn safe_url(&self) -> String {
        return self.url().map_or("<no URL>".to_string(), Url::to_string);
    }

    fn safe_status(&self) -> String {
        return self
            .status()
            .map_or("<no HTTP status>".to_string(), |status| status.to_string());
    }
}

impl Error for TfsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            TfsError::JsonParseFailed(cause, _) => Some(cause),
            TfsError::RequestTimedOut(cause) => Some(cause),
            TfsError::TfsServerError(cause) => Some(cause),
            TfsError::OtherRequestError(cause) => Some(cause),
            _ => None,
        };
    }
}

impl From<reqwest::Error> for TfsError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return TfsError::RequestTimedOut(error);
        }
        if error.is_server_error() {
            return TfsError::TfsServerError(error);
        }
        return TfsError::OtherRequestError(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tfs_response() {
        let json = r#"{"changesetId":27754,"url":"https://cqse.visualstudio.com/TestData/_apis/tfvc/changesets/27754","author":{"displayName":"CQSE GmbH","url":"https://spsprodweu1.vssps.visualstudio.com/A40e1a985-f6b9-41c2-8e31-7910c058755e/_apis/Identities/fb6b0c50-f0de-4fa2-81d0-550522b54f49","id":"fb6b0c50-f0de-4fa2-81d0-550522b54f49","uniqueName":"microsoft@cqse.eu","imageUrl":"https://cqse.visualstudio.com/_api/_common/identityImage?id=fb6b0c50-f0de-4fa2-81d0-550522b54f49"},"checkedInBy":{"displayName":"CQSE GmbH","url":"https://spsprodweu1.vssps.visualstudio.com/A40e1a985-f6b9-41c2-8e31-7910c058755e/_apis/Identities/fb6b0c50-f0de-4fa2-81d0-550522b54f49","id":"fb6b0c50-f0de-4fa2-81d0-550522b54f49","uniqueName":"microsoft@cqse.eu","imageUrl":"https://cqse.visualstudio.com/_api/_common/identityImage?id=fb6b0c50-f0de-4fa2-81d0-550522b54f49"},"createdDate":"2019-03-10T15:27:14.803Z","comment":"baseless merge v1.5 -> v2","_links":{"self":{"href":"https://cqse.visualstudio.com/TestData/_apis/tfvc/changesets/27754"},"changes":{"href":"https://cqse.visualstudio.com/_apis/tfvc/changesets/27754/changes"},"workItems":{"href":"https://cqse.visualstudio.com/_apis/tfvc/changesets/27754/workItems"},"web":{"href":"https://cqse.visualstudio.com/TestData/_versionControl/changeset/27754"},"author":{"href":"https://spsprodweu1.vssps.visualstudio.com/A40e1a985-f6b9-41c2-8e31-7910c058755e/_apis/Identities/fb6b0c50-f0de-4fa2-81d0-550522b54f49"},"checkedInBy":{"href":"https://spsprodweu1.vssps.visualstudio.com/A40e1a985-f6b9-41c2-8e31-7910c058755e/_apis/Identities/fb6b0c50-f0de-4fa2-81d0-550522b54f49"}}}"#;
        let changeset: ChangesetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(changeset.created_date, "2019-03-10T15:27:14.803Z");
    }

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(
            Tfs::parse_date("2019-03-10T15:27:14.803Z".to_string()).ok(),
            Some("1552231634000".to_string())
        );
        assert_eq!(
            Tfs::parse_date("2019-03-10T15:27:14.803-01:00".to_string()).ok(),
            Some("1552235234000".to_string())
        );
    }
}
