// std
use std::{borrow::Cow, env};
// crates.io
use anyhow::Result;
use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use subrpcer::{client::u, state};

fn main() -> Result<()> {
    Watcher::new(
        &env::var("GITHUB_TOKEN").expect("expect a GITHUB_TOKEN env var"),
        &["crab", "darwinia", "koi"],
    )
    .process()
}

struct Watcher {
    github: GitHub,
    networks: &'static [&'static str],
}
impl Watcher {
    fn new(github_token: &str, networks: &'static [&'static str]) -> Self {
        Self {
            github: GitHub::new(github_token),
            networks,
        }
    }

    fn process(&self) -> Result<()> {
        for n in self.networks {
            self.check_and_release(n)?;
        }

        Ok(())
    }

    fn check_and_release(&self, network: &str) -> Result<()> {
        let prerelease = network.starts_with("koi");
        let version_d = self
            .github
            .latest_darwinia_release_of("darwinia", None, prerelease)?;
        let version_dr = self.github.latest_darwinia_release_of(
            "darwinia-release",
            Some(network),
            prerelease,
        )?;

        println!("darwinia version        : {version_d:?}");
        println!("darwinia-release version: {version_dr:?}");

        if version_d.tag == version_dr.tag {
            println!("{network} has already included the latest upgrade");

            return Ok(());
        }

        let on_chain_version = self.on_chain_version(&if network == "darwinia" {
            Cow::Borrowed("https://rpc.darwinia.network")
        } else {
            Cow::Owned(format!("https://{network}-rpc.darwinia.network"))
        })?;

        if on_chain_version == version_d.spec {
            println!("going to release the {version_d:?} to {network}");

            self.github.release(network, &version_d.tag)?;
        } else {
            println!("{network} runtime has not been updated to the latest version yet");

            return Ok(());
        }

        Ok(())
    }

    fn on_chain_version(&self, uri: &str) -> Result<u32> {
        let ver = u::send_jsonrpc(uri, &state::get_runtime_version(0, None::<()>))?
            .into_json::<RpcResult>()?
            .result
            .spec_version;

        Ok(ver)
    }
}

#[derive(Debug)]
struct GitHub {
    token: String,
}
impl GitHub {
    fn new(token: &str) -> Self {
        Self {
            token: token.into(),
        }
    }

    fn get<T>(&self, uri: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(ureq::get(uri)
            .set("Authorization", &format!("Bearer {}", self.token))
            .call()?
            .into_json()?)
    }

    fn post<T>(&self, uri: &str, payload: T) -> Result<()>
    where
        T: Serialize,
    {
        ureq::post(uri)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/vnd.github+json")
            .send_json(payload)?;

        Ok(())
    }

    fn latest_darwinia_release_of(
        &self,
        repository: &str,
        branch: Option<&str>,
        prerelease: bool,
    ) -> Result<Version> {
        let api = format!("https://api.github.com/repos/darwinia-network/{repository}/releases");
        let releases = self.get::<Vec<GitHubRelease>>(&api)?;
        let re = if prerelease {
            Regex::new(r".*(koi-\d{4})").unwrap()
        } else {
            Regex::new(r".*(v\d+\.\d+\.\d+(-\d+)?)").unwrap()
        };

        for r in releases {
            if prerelease != r.prerelease {
                continue;
            }
            if branch.map_or(true, |b| r.target_commitish == b) {
                let tag = re
                    .captures(&r.tag_name)
                    .and_then(|c| c.get(1).map(|m| m.as_str()))
                    .unwrap_or_else(|| panic!("invalid tag name {}", r.tag_name));
                let spec = tag2spec_version(tag, prerelease)?;

                return Ok(Version {
                    spec,
                    tag: tag.into(),
                });
            }
        }

        Ok(Version {
            spec: 0,
            tag: "".into(),
        })
    }

    fn release(&self, network: &str, tag: &str) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct Payload {
            r#ref: String,
            inputs: Inputs,
        }
        #[derive(Debug, Serialize)]
        struct Inputs {
            network: String,
            tag: String,
        }

        let api = "https://api.github.com/repos/darwinia-network/darwinia-release/actions/workflows/node.yml/dispatches";
        let payload = Payload {
            r#ref: "main".into(),
            inputs: Inputs {
                network: network.into(),
                tag: tag.into(),
            },
        };

        self.post(api, payload)
    }
}

#[derive(Debug, Deserialize)]
struct RpcResult {
    result: OnChainVersion,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnChainVersion {
    spec_version: u32,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    prerelease: bool,
    tag_name: String,
    target_commitish: String,
}

#[derive(Debug)]
struct Version {
    spec: u32,
    tag: String,
}

fn tag2spec_version(tag: &str, prerelease: bool) -> Result<u32> {
    if prerelease {
		// TODO: accept dynamic size.
        Ok(tag[4..].parse()?)
    } else {
        let tag = tag[1..].split('.').collect::<Vec<_>>();
        let mut ver = tag[0].parse::<u32>()? * 1_000 + tag[1].parse::<u32>()? * 100;
        let last_part = tag.last().unwrap().split('-').collect::<Vec<_>>();

        ver += last_part[0].parse::<u32>()? * 10;

        if last_part.len() == 2 {
            ver += last_part[1].parse::<u32>()?;
        }

        Ok(ver)
    }
}

#[test]
fn tag2spec_version_should_work() {
    assert_eq!(tag2spec_version("v1.0.0", false).unwrap(), 1000);
    assert_eq!(tag2spec_version("v1.2.0", false).unwrap(), 1200);
    assert_eq!(tag2spec_version("v1.2.3", false).unwrap(), 1230);
    assert_eq!(tag2spec_version("v1.2.3-1", false).unwrap(), 1231);
    assert_eq!(tag2spec_version("koi-1234", true).unwrap(), 1234);
}

#[test]
fn latest_darwinia_release_of_should_work() {
    let github = GitHub::new(&env::var("GITHUB_TOKEN").unwrap());
    let version = github
        .latest_darwinia_release_of("darwinia", None, false)
        .unwrap();

    println!("latest release of darwinia is {version:?}");

    let version = github
        .latest_darwinia_release_of("darwinia", None, true)
        .unwrap();

    println!("latest prerelease of darwinia is {version:?}");

    let version = github
        .latest_darwinia_release_of("darwinia-release", Some("crab"), false)
        .unwrap();

    println!("latest release of darwinia-release is {version:?}");

    let version = github
        .latest_darwinia_release_of("darwinia-release", Some("darwinia"), false)
        .unwrap();

    println!("latest release of darwinia-release is {version:?}");

    let version = github
        .latest_darwinia_release_of("darwinia-release", Some("koi"), true)
        .unwrap();

    println!("latest prerelease of darwinia-release is {version:?}");
}
