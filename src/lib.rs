//This file is part of metacpan_api
//
//metacpan_api is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//metacpan_api is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with Foobar.  If not, see <http://www.gnu.org/licenses/>.

extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Http(reqwest::Error),
    #[fail(display = "{}", _0)]
    Url(url::ParseError),
    #[fail(display = "Not found")]
    NotFound,
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::Url(e)
    }
}

pub struct SyncClient {
    client: reqwest::Client,
    base_url: Url,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    #[serde(rename = "type")]
    pub repo_type: Option<String>,
    pub web: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Resources {
    pub homepage: Option<String>,
    pub repository: Option<Repository>,
}

#[derive(Deserialize, Debug)]
pub struct PerlDep {
    pub module: String,
    pub phase: String,
    pub relationship: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct PerlInfo {
    pub dependency: Option<Vec<PerlDep>>,
    #[serde(rename = "abstract")]
    pub description: Option<String>,
    pub download_url: String,
    pub license: Option<Vec<String>>,
    #[serde(rename = "distribution")]
    pub name: String,
    pub resources: Resources,
    pub version: serde_json::value::Value,
}

#[derive(Deserialize, Debug)]
struct PerlModule {
    distribution: String,
}

impl SyncClient {
    /// Instantiate a new synchronous API client.
    ///
    /// This will fail if the underlying http client could not be created.
    pub fn new() -> Self {
        SyncClient {
            client: reqwest::Client::new(),
            base_url: Url::parse("https://fastapi.metacpan.org/v1/").unwrap(),
        }
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        info!("GET {}", url);

        let mut res = {
            let res = self.client.get(url).send()?;

            if res.status() == StatusCode::NOT_FOUND {
                return Err(Error::NotFound);
            }
            res.error_for_status()?
        };

        let data: T = res.json()?;
        Ok(data)
    }

    pub fn perl_info(&self, name: &str) -> Result<PerlInfo, Error> {
        let url = self.base_url.join(&("release/".to_string() + &name.replace("::", "-")))?;
        let data: PerlInfo = self.get(url)?;

        let deserialized_resources = Resources {
            homepage: data.resources.homepage,
            repository: data.resources.repository,
        };

        Ok(PerlInfo {
            dependency: data.dependency,
            description: data.description,
            download_url: data.download_url,
            license: data.license,
            name: data.name,
            resources: deserialized_resources,
            version: serde_json::Value::String(data.version.to_string()),
        })
    }

    /// Takes the name of a module and returns the name of the distribution
    /// it's in
    pub fn get_dist(&self, name: &str) -> Result<String, Error> {
        let url = self.base_url.join(&("module/".to_string() + &name))?;
        let data: PerlModule = self.get(url)?;

        Ok(data.distribution)
    }
}

#[cfg(test)]
mod test {
    use SyncClient;

    #[test]
    fn test_name() {
        let client = SyncClient::new();
        let perl_info = client.perl_info("Moose");
        assert_eq!(perl_info.unwrap().name, "Moose");
    }

    #[test]
    fn test_db_point_names() {
        let client = SyncClient::new();
        let perl_info = client.perl_info("JSON::PP");
        assert!(perl_info.unwrap().name.len() > 0);
    }

    #[test]
    fn query_module() {
        let client = SyncClient::new();
        let perl_info = client.perl_info(&client.get_dist("Scalar::Util").unwrap());
        assert_eq!(perl_info.unwrap().name, "Scalar-List-Utils");
    }

    #[test]
    fn float_ver() {
        let client = SyncClient::new();
        client.perl_info("Dist-Zilla-Plugin-Conflicts").unwrap();
    }
}
