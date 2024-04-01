// std
use std::{env, error::Error, process};
// crates.io
use serde::{Deserialize, Serialize};
use subrpcer::{client::u, state};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let w = Watcher {
        github_token: env::var("GITHUB_TOKEN").expect("expect a GITHUB_TOKEN env var"),
        networks: &["crab", "darwinia", "pangolin"],
    };

    w.process()?;

    Ok(())
}

struct Watcher {
    github_token: String,
    networks: &'static [&'static str],
}
impl Watcher {
    fn process(&self) -> Result<()> {
        for n in self.networks {
            #[allow(clippy::type_complexity)]
            let f: Box<dyn Fn(&str) -> Result<(String, u32)>> = match *n {
                "crab" | "darwinia" => Box::new(|r| self.github_release_version_of(r)),
                "pangolin" => Box::new(|r| self.github_pre_release_version_of(r)),
                _ => unreachable!(),
            };

            self.check_and_release(n, f)?;
        }

        Ok(())
    }

    fn check_and_release<F>(&self, network: &str, github_version_of: F) -> Result<()>
    where
        F: Fn(&str) -> Result<(String, u32)>,
    {
        let (tag, github_version_d) = github_version_of("darwinia")?;
        let (_, github_version_dr) = github_version_of("darwinia-release")?;

        if github_version_d == github_version_dr {
            println!("we already have the latest version");

            process::exit(-1);
        }

        let on_chain_version = self.on_chain_version(network)?;

        if on_chain_version == github_version_d {
            self.release(network, &tag)?;
        } else {
            println!("runtime has not been updated to the latest version yet");

            process::exit(-1);
        }

        Ok(())
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

        let response = ureq::post("https://api.github.com/repos/darwinia-network/darwinia-release/actions/workflows/node.yml/dispatches")
            .set("Authorization", &format!("Bearer {}", self.github_token))
            .set("Accept", "application/vnd.github+json")
            .send_json(Payload {
                r#ref: "main".into(),
                inputs: Inputs {
                    network: network.into(),
                    tag: tag.into(),
                },
            })?;

        Ok(())
    }

    fn github_release_version_of(&self, repository: &str) -> Result<(String, u32)> {
        let tag = ureq::get(&format!(
            "https://api.github.com/repos/darwinia-network/{repository}/releases/latest"
        ))
        .set("Authorization", &format!("Bearer {}", self.github_token))
        .call()?
        .into_json::<GithubReleaseVersion>()?
        .tag_name;
        let ver = tag2spec_version(&tag)?;

        Ok((tag, ver))
    }

    fn github_pre_release_version_of(&self, repository: &str) -> Result<(String, u32)> {
        let releases = ureq::get(&format!(
            "https://api.github.com/repos/darwinia-network/{repository}/releases"
        ))
        .set("Authorization", &format!("Bearer {}", self.github_token))
        .call()?
        .into_json::<Vec<GithubReleaseVersion>>()?;
        let tag = releases
            .into_iter()
            .find(|release| release.prerelease)
            .ok_or("no pre-release found")?
            .tag_name;
        let ver = tag[6..].parse()?;

        Ok((tag, ver))
    }

    fn on_chain_version(&self, network: &str) -> Result<u32> {
        let ver = u::send_jsonrpc(
            &format!("https://{network}-rpc.darwiniacommunitydao.xyz"),
            &state::get_runtime_version(0, None::<()>),
        )?
        .into_json::<RpcResult>()?
        .result
        .spec_version;

        Ok(ver)
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
struct GithubReleaseVersion {
    tag_name: String,
    prerelease: bool,
}

fn tag2spec_version(tag: &str) -> Result<u32> {
    let tag = tag[1..].split('.').collect::<Vec<_>>();
    let mut ver = tag[0].parse::<u32>()? * 1_000 + tag[1].parse::<u32>()? * 100;
    let last_part = tag.last().unwrap().split('-').collect::<Vec<_>>();

    ver += last_part[0].parse::<u32>()? * 10;

    if last_part.len() == 2 {
        ver += last_part[1].parse::<u32>()?;
    }

    Ok(ver)
}

#[test]
fn tag2spec_version_should_work() {
    assert_eq!(tag2spec_version("v1.0.0").unwrap(), 1000);
    assert_eq!(tag2spec_version("v1.2.0").unwrap(), 1200);
    assert_eq!(tag2spec_version("v1.2.3").unwrap(), 1230);
    assert_eq!(tag2spec_version("v1.2.3-1").unwrap(), 1231);
}
