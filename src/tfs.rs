use std::error::Error;

use reqwest::{Response, Url};
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use crate::app::App;
use crate::utils::PeekOption;

/// Struct for retrieving info from a TFVC repo.
pub struct Tfs<'a> {
    app: &'a App,
}

#[derive(Deserialize)]
struct ChangesetResponse {
    origin: String,
}

enum TfsError {}

impl<'a> Tfs<'a> {
    pub fn guess_timestamp(&self) -> Option<String> {
        let teamproject = self.app.env_variable("SYSTEM_TEAMPROJECTID")?;
        let changeset = self.app.env_variable("BUILD_SOURCEVERSION")?;
        let collection_uri = self.app.env_variable("SYSTEM_TEAMFOUNDATIONCOLLECTIONURI")?;

        let access_token = self.get_access_token()?;
        let url = self.create_changeset_url(collection_uri, teamproject, changeset)?;

        let response = self.request(url, access_token)?;
        let changeset_response = self.parse_response(response);
        None
    }

    fn parse_response(&self, mut response: Response) -> Option<ChangesetResponse> {
        return match response.json::<ChangesetResponse>() {
            Ok(json) => Some(json),
            Err(error) => {
                self.app.log(&format!("Failed to parse JSON response from TFS: {}",
                                      error.description()));
                None
            }
        }
    }

    fn request(&self, url: Url, access_token: String) -> Option<Response> {
        let url_string = url.to_string();

        let client = reqwest::Client::new();
        let result = client.get(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send();
        return match result {
            Ok(response) => Some(response),
            Err(error) => {
                self.app.log(&format!("Request to {} failed: {}", url_string, self.describe_error(&error)));
                None
            }
        };
    }

    fn describe_error(&self, error: &reqwest::Error) -> String {
        if error.is_timeout() {
            return format!("Request timed out: {}", error.description());
        }
        return match error.status() {
            Some(status) => format!("Failed with HTTP status code {}: {}",
                                    status.as_str(), error.description()),
            None => error.description().to_string(),
        };
    }

    fn create_changeset_url(&self, collection_uri: String, teamproject: String, changeset: String) -> Option<Url> {
        let url_string = &format!("{}/{}/_apis/tfvc/changesets/{}",
                                  collection_uri, teamproject, changeset);
        return match Url::parse(url_string) {
            Ok(url) => Some(url),
            Err(error) => {
                self.app.log(&format!("Failed to parse {} as a url: {}", url_string,
                                      error.description()));
                None
            }
        };
    }

    fn get_access_token(&self) -> Option<String> {
        return self.app.env_variable("SYSTEM_ACCESSTOKEN").if_none(|| {
            self.app.log("Environment variable SYSTEM_ACCESSTOKEN not set. Please make sure \
                you activated 'Additional options > Allow scripts to access OAuth token' for your \
                pipeline job! Otherwise, the timestamp for a TFVC changeset cannot be determined.");
        });
    }
}
