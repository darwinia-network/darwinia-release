import os, sys

import requests


def get_spec_version(chain: str) -> int:
    response = requests.post(
        f"https://{chain}-rpc.darwiniacommunitydao.xyz",
        headers={"Content-Type": "application/json"},
        json={
            "jsonrpc": "2.0",
            "id": 0,
            "method": "state_getRuntimeVersion",
            "params": [],
        },
    )

    if response.status_code == 200:
        try:
            return int(response.json()["result"]["specVersion"])
        except TypeError:
            print("a value in the JSON object is not a dictionary or list as expected")
            print(response.text)
            sys.exit(-1)
    else:
        print(f"error while executing query: {response.status_code}, {response.text}")
        sys.exit(-1)


def get_latest_release():
    response = requests.get(
        "https://api.github.com/repos/darwinia-network/darwinia/releases",
    )

    if response.status_code == 200:
        releases = response.json()

        for release in releases:
            if not release["prerelease"]:
                t = release["tag_name"]

                return t

        print("no non-pre-release tag found")
        sys.exit(-1)

    else:
        print(f"error while executing query: {response.status_code}, {response.text}")
        sys.exit(-1)


def tag2spec_version(tag: str) -> str:
    parts = tag[1:].split(".")
    last_part = parts[-1].split("-")

    parts[-1:] = last_part

    if len(last_part) < 2:
        parts.append("0")

    return "".join(parts)


# def dispatch_release(github_token: str, network: str):
#     response = requests.post(
#         "https://api.github.com/repos/darwinia-network/darwinia-release/dispatches",
#         headers={
#             "Authorization": f"token {github_token}",
#             "Accept": "application/vnd.github.everest-preview+json",
#         },
#         json={
#             "event_type": "node",
#             "client_payload": {"network": network},
#         },
#     )
#
#     if response.status_code == 204:
#         print("workflow dispatch event created successfully")
#     else:
#         print(
#             f"error while creating workflow dispatch event: {response.status_code}, {response.text}"
#         )


crab_csv = get_spec_version("darwinia")
darwinia_csv = get_spec_version("darwinia")
release_sv = tag2spec_version(get_latest_release())

print(f"Crab chain spec version: {crab_csv}")
print(f"Darwinia chain spec version: {darwinia_csv}")
print(f"release spec version: {release_sv}")

gh_tk = os.getenv("GITHUB_TOKEN")

if gh_tk is None:
    print("`GITHUB_TOKEN` ENV var is not set")
    sys.exit(-1)
if crab_csv != release_sv:
    pass
    # dispatch_release(gh_tk, "crab")
if darwinia_csv != release_sv:
    pass
    # dispatch_release(gh_tk, "darwinia")
