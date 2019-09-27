use std::error::Error;

use chrono::DateTime;
use reqwest::header::AUTHORIZATION;
use reqwest::{Response, Url};
use serde::Deserialize;

use crate::app::App;
use crate::utils::PeekOption;

/// Struct for retrieving info from a TFVC repo.
pub struct Tfs<'a> {
    app: &'a App,
}

#[derive(Deserialize)]
struct ChangesetResponse {
    #[serde(rename = "createdDate")]
    created_date: String,
}

impl<'a> Tfs<'a> {
    pub fn new(app: &'a App) -> Tfs<'a> {
        return Tfs { app };
    }

    pub fn guess_timestamp(&self) -> Option<String> {
        let teamproject = self.app.env_variable("SYSTEM_TEAMPROJECTID")?;
        let changeset = self.app.env_variable("BUILD_SOURCEVERSION")?;
        let collection_uri = self
            .app
            .env_variable("SYSTEM_TEAMFOUNDATIONCOLLECTIONURI")?;

        let access_token = self.get_access_token()?;
        let url = self.create_changeset_url(collection_uri, teamproject, changeset)?;

        let response = self.request(url, access_token)?;
        let changeset_response = self.parse_response(response)?;
        return Tfs::parse_date(changeset_response.created_date);
    }

    fn parse_date(date_string: String) -> Option<String> {
        return DateTime::parse_from_rfc3339(&date_string)
            .map(|date| format!("{}000", date.timestamp()))
            .ok();
    }

    fn parse_response(&self, mut response: Response) -> Option<ChangesetResponse> {
        return match response.json::<ChangesetResponse>() {
            Ok(json) => Some(json),
            Err(error) => {
                self.app.log(&format!(
                    "Failed to parse JSON response from TFS: {}",
                    error.description()
                ));
                None
            }
        };
    }

    fn request(&self, url: Url, access_token: String) -> Option<Response> {
        let url_string = url.to_string();

        let client = reqwest::Client::new();
        let result = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send();
        return match result {
            Ok(response) => Some(response),
            Err(error) => {
                self.app.log(&format!(
                    "Request to {} failed: {}",
                    url_string,
                    self.describe_error(&error)
                ));
                None
            }
        };
    }

    fn describe_error(&self, error: &reqwest::Error) -> String {
        if error.is_timeout() {
            return format!("Request timed out: {}", error.description());
        }
        return match error.status() {
            Some(status) => format!(
                "Failed with HTTP status code {}: {}",
                status.as_str(),
                error.description()
            ),
            None => error.description().to_string(),
        };
    }

    fn create_changeset_url(
        &self,
        collection_uri: String,
        teamproject: String,
        changeset: String,
    ) -> Option<Url> {
        let url_string = &format!(
            "{}/{}/_apis/tfvc/changesets/{}",
            collection_uri, teamproject, changeset
        );
        return match Url::parse(url_string) {
            Ok(url) => Some(url),
            Err(error) => {
                self.app.log(&format!(
                    "Failed to parse {} as a url: {}",
                    url_string,
                    error.description()
                ));
                None
            }
        };
    }

    fn get_access_token(&self) -> Option<String> {
        return self.app.env_variable("SYSTEM_ACCESSTOKEN").if_none(|| {
            self.app.log(
                "Environment variable SYSTEM_ACCESSTOKEN not set. Please make sure \
                 you activated 'Additional options > Allow scripts to access OAuth token' for your \
                 pipeline job! Otherwise, the timestamp for a TFVC changeset cannot be determined.",
            );
        });
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
            Tfs::parse_date("2019-03-10T15:27:14.803Z".to_string()),
            Some("1552231634000".to_string())
        );
        assert_eq!(
            Tfs::parse_date("2019-03-10T15:27:14.803-01:00".to_string()),
            Some("1552235234000".to_string())
        );
    }

    #[test]
    fn test_accessing_azure_devops_api() {
        let app = App::new(true, |env_variable| Some("not-needed".to_string()));
        let tfs = Tfs::new(&app);
        assert_eq!(
            tfs.fetch_changeset_creation_date(
                "https://cqse.visualstudio.com/".to_string(),
                "TestData".to_string(),
                "27754".to_string(),
                "jc6vthfrnu2myipy2nbqdrxwq62qoyy2qbph65onddalu5ixge6a".to_string(),
            ),
            Some("1552231634000".to_string())
        )
    }
}
