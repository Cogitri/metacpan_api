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

// TODO: parsing dependencies doesn't work as of now
/*
#[derive(Deserialize)]
pub struct Deps {
    #[serde(untagged)]
    pub dep: String,
}

#[derive(Deserialize)]
pub struct PerlDepDetail {
    pub suggests: Option<Vec<String>>,
    pub recommends: Option<Vec<String>>,
    pub requires: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct PerlDeps {
    pub runtime: PerlDepDetail,
    pub develop: PerlDepDetail,
    pub test: PerlDepDetail,
    pub configure: PerlDepDetail,
}
*/

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
    pub repository: Repository,
}

#[derive(Deserialize, Debug)]
pub struct PerlInfo {
    pub name: String,
    #[serde(rename = "abstract")]
    pub description: Option<String>,
    pub version: String,
    pub license: Vec<String>,
    pub resources: Resources,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub metadata: PerlInfo,
}

impl SyncClient {
    /// Instantiate a new synchronous API client.
    ///
    /// This will fail if the underlying http client could not be created.
    pub fn new() -> Self {
        SyncClient {
            client: reqwest::Client::new(),
            base_url: Url::parse("https://fastapi.metacpan.org/v1/release/").unwrap(),
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
        let url = self.base_url.join(&name)?;
        let data: Data = self.get(url)?;

        let deserialized_resources = Resources {
            homepage: data.metadata.resources.homepage,
            repository: Repository {
                repo_type: data.metadata.resources.repository.repo_type,
                web: data.metadata.resources.repository.web,
                url: data.metadata.resources.repository.url,
            }
        };

        /*
        let deserialized_deps = PerlDeps {
            runtime: PerlDepDetail {
                requires: data.metadata.dependencies.runtime.requires,
                recommends: data.metadata.dependencies.runtime.recommends,
                suggests: data.metadata.dependencies.runtime.suggests,
            },
            test: PerlDepDetail {
                requires: data.metadata.dependencies.test.requires,
                recommends: data.metadata.dependencies.test.recommends,
                suggests: data.metadata.dependencies.test.suggests,
            },
            configure: PerlDepDetail {
                requires: data.metadata.dependencies.configure.requires,
                recommends: data.metadata.dependencies.configure.recommends,
                suggests: data.metadata.dependencies.configure.suggests,
            },
            develop: PerlDepDetail {
                requires: data.metadata.dependencies.develop.requires,
                recommends: data.metadata.dependencies.develop.recommends,
                suggests: data.metadata.dependencies.develop.suggests,
            }
        };
        */

        Ok(PerlInfo{
            name: data.metadata.name,
            description: data.metadata.description,
            version: data.metadata.version,
            license: data.metadata.license,
            resources: deserialized_resources,
        })
    }
}

#[cfg(test)]
mod test {
    use SyncClient;

    #[test]
    fn test_name() {
        let client = SyncClient::new();
        let perl_info = client.perl_info("Moose");
        assert!(perl_info.unwrap().name.len() > 0);
    }
}