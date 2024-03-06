from datetime import datetime
import os, subprocess, sys

tag = sys.argv[1]
runtime = sys.argv[2]
features = sys.argv[3]
features_ = features.replace(",", "-")
cmd = [
    "rtor",
    "-n",
    "-g",
    "https://github.com/darwinia-network/darwinia",
    "-t",
    tag,
    "-m",
    f"runtime/{runtime}/Cargo.toml",
    "-r",
    runtime,
    "-f",
    features,
    "-o",
    f"wasm/{features_}",
]

print(" ".join(cmd))
subprocess.run(cmd).check_returncode()

dir = f"wasm/{features_}"
os.makedirs(dir, exist_ok=True)

with open(f"{dir}/CHANGELOG", "a+") as f:
    f.write(f'Generated {tag} at {datetime.now().strftime("%m/%d/%Y %H:%M:%S")}\n')
