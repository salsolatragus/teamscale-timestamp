use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Read;

use chrono::DateTime;
use reqwest::{RedirectPolicy, RequestBuilder, Response, StatusCode, Url};
use serde::Deserialize;

use crate::env_reader::EnvReader;
use crate::logger::Logger;

/// Retries info from a TFVC repo.
pub struct Tfs<'a> {
    logger: &'a Logger,
    env_reader: &'a EnvReader<'a>,
}

/// The type of access token to use when connecting to the TFS.
enum AccessToken {
    Personal(String),
    Oauth(String),
}

/// Represents the authentication method and data used to communicate with the TFS.
impl AccessToken {
    fn configure(&self, request_builder: RequestBuilder) -> RequestBuilder {
        return match self {
            AccessToken::Personal(token) => request_builder.basic_auth("", Some(token)),
            AccessToken::Oauth(token) => request_builder.bearer_auth(token),
        };
    }
}

/// JSON response from the TFS for a changeset.
#[derive(Deserialize)]
struct ChangesetResponse {
    #[serde(rename = "createdDate")]
    created_date: String,
}

impl<'a> Tfs<'a> {
    pub fn new(logger: &'a Logger, env_reader: &'a EnvReader) -> Tfs<'a> {
        return Tfs { logger, env_reader };
    }

    /// Guesses the timestamp to which to upload the external analysis result based on the
    /// changeset reported by the TFS. Does a network request to determine the changeset's
    /// creation time.
    pub fn timestamp(&self, personal_access_token: Option<&str>) -> Option<String> {
        let teamproject = self.env_reader.env_variable("SYSTEM_TEAMPROJECTID")?;
        let changeset = self.env_reader.env_variable("BUILD_SOURCEVERSION")?;
        let collection_uri = self
            .env_reader
            .env_variable("SYSTEM_TEAMFOUNDATIONCOLLECTIONURI")?;
        return match self.timestamp_or_error(
            collection_uri,
            teamproject,
            changeset,
            personal_access_token,
        ) {
            Ok(timestamp) => Some(timestamp),
            Err(error) => {
                self.logger.log(&format!("{}", error));
                None
            }
        };
    }

    fn timestamp_or_error(
        &self,
        collection_uri: String,
        teamproject: String,
        changeset: String,
        personal_access_token: Option<&str>,
    ) -> TfsResult<String> {
        let url = self.create_changeset_url(collection_uri, teamproject, changeset);
        let access_token = match personal_access_token {
            Some(token) => AccessToken::Personal(token.to_string()),
            None => AccessToken::Oauth(self.get_access_token()?),
        };
        let response = self.request(url, access_token)?;
        let changeset_response = self.parse_response(response)?;
        return parse_date(changeset_response.created_date);
    }

    fn parse_response(&self, mut response: Response) -> TfsResult<ChangesetResponse> {
        let mut string = String::new();
        response
            .read_to_string(&mut string)
            .map_err(TfsError::CannotReadRequestBody)?;
        return serde_json::from_str::<ChangesetResponse>(&string)
            .map_err(|error| TfsError::JsonParseFailed(error, string));
    }

    fn request(&self, url: Url, access_token: AccessToken) -> TfsResult<Response> {
        self.logger.log(format!("Requesting URL {}", url));
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .redirect(RedirectPolicy::none())
            .build()
            .unwrap();
        let response = access_token.configure(client.get(url)).send()?;

        if is_tfs_signin_redirect(&response) {
            return Err(TfsError::InvalidAccessToken());
        }
        if !response.status().is_success() {
            return Err(TfsError::RequestStatusNotSuccessful(response));
        }
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
            .env_reader
            .env_variable("SYSTEM_ACCESSTOKEN")
            .ok_or(TfsError::AccessTokenNotProvided());
    }
}

/// If the used credentials are invalid, the TFS sends a 302 status code and redirects the user
/// to the _signin page.
fn is_tfs_signin_redirect(response: &Response) -> bool {
    if let Some(location) = response
        .headers()
        .get(reqwest::header::LOCATION)
        .map(|header| header.to_str())
        .transpose()
        .unwrap_or(None)
    {
        return response.status() == StatusCode::FOUND && location.contains("/_signin");
    }
    return false;
}

fn parse_date(date_string: String) -> TfsResult<String> {
    return DateTime::parse_from_rfc3339(&date_string)
        .map(|date| format!("{}{:03}", date.timestamp(), date.timestamp_subsec_millis()))
        .map_err(|error| TfsError::DateStringCannotBeParsed(error, date_string));
}

/// All errors that can occurr when trying to determine the timestamp of a TFVC changeset.
#[derive(Debug)]
enum TfsError {
    JsonParseFailed(serde_json::error::Error, String),
    RequestTimedOut(reqwest::Error),
    CannotReadRequestBody(std::io::Error),
    TfsInternalServerError(reqwest::Error),
    AccessTokenNotProvided(),
    InvalidAccessToken(),
    RequestStatusNotSuccessful(Response),
    OtherRequestError(reqwest::Error),
    DateStringCannotBeParsed(chrono::format::ParseError, String),
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
            TfsError::TfsInternalServerError(cause) => write!(
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
                 job, which will set the correct SYSTEM_ACCESSTOKEN environment variable! \
                 Otherwise, the timestamp for a TFVC changeset cannot be determined. \
                 Alternatively, you can provide a personal access token via the command line option \
                 --tfs-pat."
            ),
            TfsError::DateStringCannotBeParsed(cause, date_string) => {
                write!(f, "TFS returned unparsable date {}: {}", date_string, cause)
            }
            TfsError::CannotReadRequestBody(cause) => {
                write!(f, "Failed to read request body: {}", cause)
            }
            TfsError::RequestStatusNotSuccessful(response) => write!(
                f,
                "Request to {} failed with status code {}",
                response.url(),
                response.status()
            ),
        }
    }
}

type TfsResult<T> = std::result::Result<T, TfsError>;

impl Error for TfsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            TfsError::JsonParseFailed(cause, _) => Some(cause),
            TfsError::RequestTimedOut(cause) => Some(cause),
            TfsError::TfsInternalServerError(cause) => Some(cause),
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
            return TfsError::TfsInternalServerError(error);
        }
        return TfsError::OtherRequestError(error);
    }
}

/// Wrappers around optional properties of a reqwest error.
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
            parse_date("2019-03-10T15:27:14.003Z".to_string()).ok(),
            Some("1552231634003".to_string())
        );
        assert_eq!(
            parse_date("2019-03-10T15:27:14.803-01:00".to_string()).ok(),
            Some("1552235234803".to_string())
        );
    }

    ///#[test]
    fn test_request() {
        let access_token = std::env::var("TFS_ACCESS_TOKEN").unwrap();
        let logger = Logger::new(true);
        let env_reader = EnvReader::new(|_| None);
        let tfs = Tfs::new(&logger, &env_reader);
        let result = tfs.timestamp_or_error(
            "https://cqse.visualstudio.com".to_string(),
            "TestData".to_string(),
            "27754".to_string(),
            Some(access_token.as_str()),
        );
        assert_eq!(result.unwrap(), "1552231634803".to_string());
    }

    #[test]
    fn test_invalid_access_token() {
        let logger = Logger::new(true);
        let env_reader = EnvReader::new(|_| None);
        let tfs = Tfs::new(&logger, &env_reader);
        let result = tfs.timestamp_or_error(
            "https://cqse.visualstudio.com".to_string(),
            "TestData".to_string(),
            "27754".to_string(),
            Some("invalid"),
        );

        let error = result.err();
        match error {
            Some(TfsError::InvalidAccessToken()) => (),
            _ => panic!("incorrect error type: {:?}", error),
        }
    }
}
