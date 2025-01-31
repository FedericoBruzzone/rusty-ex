import subprocess
import time
import os
import json
import sys
import requests
import toml
import shutil
from datetime import datetime
import signal

def printt(message):
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {message}")

# get the number of stars of a GitHub repository
def get_github_stars(repo_url):
    try:
        repo_path = repo_url.split("github.com/")[-1].replace(".git", "")
        api_url = f"https://api.github.com/repos/{repo_path}"
        response = requests.get(api_url)
        response.raise_for_status()
        data = response.json()
        return data.get("stargazers_count", 0)
    except Exception as e:
        printt(f"Error fetching GitHub stars: {e}")
        return None

# get the number of downloads of a crate from crates.io
def get_cratesio_downloads(crate_name):
    try:
        api_url = f"https://crates.io/api/v1/crates/{crate_name}"
        response = requests.get(api_url)
        response.raise_for_status()
        data = response.json()
        return data.get("crate", {}).get("downloads", 0)
    except Exception as e:
        printt(f"Error fetching crates.io downloads: {e}")
        return None

# return and parse Cargo.toml file
def parse_cargo_toml(subdir="."):
    cargo_toml = os.path.join(subdir, "Cargo.toml")

    if os.path.exists(cargo_toml):
        try:
            with open(cargo_toml, "r") as f:
                return toml.load(f)
        except Exception as e:
            printt(f"Error reading Cargo.toml: {e}")

# analyze the folder at `repo_name` and return a list of formatted LaTeX rows
def analyze(crate_name, repo_url, repo_name, stars, downloads, reset_cargo=False):
    printt(f"Analyzing {repo_name}.")

    os.chdir(repo_name)

    cargo_toml = parse_cargo_toml(".")
    loc = 0
    for root, _, files in os.walk("."):
        for file in files:
            if file.endswith(".rs"):
                result = subprocess.run(["wc", "-l", os.path.join(root, file)], capture_output=True, text=True)
                loc += int(result.stdout.split()[0])

    if reset_cargo:
        try:
            os.rename("Cargo.toml", "Cargo.toml.bak")
            subprocess.run(["cargo", "init", "--name", "temp"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except Exception as e:
            printt(f"Error resetting Cargo.toml: {e}")
            os.chdir("..")
            return []
    subprocess.run(["cargo", "clean"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    start_time = time.time()

    try:
        process = subprocess.Popen(["cargo-rustc-ex", "--print-metadata"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, preexec_fn=os.setsid)
        stdout, stderr = process.communicate(timeout=600) # 10 minutes timeout
        # stdout, stderr = process.communicate(timeout=10) # test timeout
    except subprocess.TimeoutExpired:
        os.killpg(os.getpgid(process.pid), signal.SIGTERM)
        printt(f"Error: {repo_name} timed out.")
        os.chdir("..")
        return []

    execution_time = time.time() - start_time

    results = []
    for line in stdout.decode().splitlines():
        if not line.strip(): continue
        try:
            parsed_line = json.loads(line)

            results.append({
                "url": repo_url,
                "crate": crate_name,
                "member": repo_name,
                "github_stars": stars,
                "cratesio_downloads": downloads,
                "lines_of_code": loc,
                "dependencies": len(cargo_toml.get("dependencies", {})),
                "defined_features": len(cargo_toml.get("features", {})),
                "ast_nodes": parsed_line.get("ast_nodes", "N/A"),
                "ast_edges": parsed_line.get("ast_edges", "N/A"),
                "ast_height": parsed_line.get("ast_height", "N/A"),
                "feature_nodes": parsed_line.get("features_nodes", "N/A"),
                "feature_edges": parsed_line.get("features_edges", "N/A"),
                "artifact_nodes": parsed_line.get("artifacts_nodes", "N/A"),
                "artifact_edges": parsed_line.get("artifacts_edges", "N/A"),
                "execution_time": execution_time,
                "peak_memory_usage": None,
            })

        except json.JSONDecodeError:
            printt(f"Error: Line is not a valid JSON.")
            continue

    if reset_cargo:
        try:
            os.remove("Cargo.toml")
            os.rename("Cargo.toml.bak", "Cargo.toml")
        except Exception as e:
            printt(f"Error resetting Cargo.toml: {e}")

    os.chdir("..")
    return results

# launch the analysis for each member of the workspace
def analyze_members(repo_name, repo_url, members, stars, downloads):
    res = []
    for member in members:
        try:
            if not os.path.exists(f"{repo_name}/{member}"):
                printt(f"Error: Member {member} does not exist.")
                continue

            base = os.path.basename(member)

            if os.path.exists(base):
                shutil.rmtree(base)

            shutil.move(f"{repo_name}/{member}", ".")
            res += analyze(repo_name, repo_url, base, stars, downloads, True)
            shutil.move(base, repo_name)

        except Exception as e:
            printt(f"Error analyzing member {member}: {e}")
            continue

    return res

# format the results as a LaTeX row
def format(res):
    def g(x): return res.get(x, "N/A")
    return f"\\href{{{g('url')}}}{{\\underline{{{g('crate')}}}}} & {g('member')} & {g('github_stars')} & {g('cratesio_downloads')} & {g('lines_of_code')} & {g('dependencies')} & {g('defined_features')} & {g('ast_nodes')} & {g('ast_edges')} & {g('ast_height')} & {g('feature_nodes')} & {g('feature_edges')} & {g('artifact_nodes')} & {g('artifact_edges')} & {g('execution_time'):.2f}s & {g('peak_memory_usage')} \\\\ \\hline"

def main(repo_url):
    try:
        repo_name = os.path.basename(repo_url).replace(".git", "")
        stars = get_github_stars(repo_url)
        downloads = get_cratesio_downloads(repo_name)

        if os.path.exists(repo_name):
            shutil.rmtree(repo_name)

        printt(f"Cloning the repository from {repo_url}.")
        subprocess.run(["git", "clone", "--recurse-submodules", "-j8", repo_url], check=True)

        cargo_toml = parse_cargo_toml(repo_name)
        members = cargo_toml.get("workspace", {}).get("members", [])

        # expand members with wildcard
        expanded_members = []
        for member in members:
            if member.endswith("/*"):
                member_dir = os.path.join(repo_name, member[:-2])
                if os.path.exists(member_dir):
                    expanded_members.extend([os.path.join(member[:-2], subdir) for subdir in os.listdir(member_dir) if os.path.isdir(os.path.join(member_dir, subdir))])
                else:
                    printt(f"Warning: Directory {member_dir} does not exist.")
            else:
                expanded_members.append(member)
        members = expanded_members

        printt(f"Members: {members}")

        results = analyze(repo_name, repo_url, repo_name, stars, downloads)
        results += analyze_members(repo_name, repo_url, members, stars, downloads)

        print("Crate, Member, GH Stars, Crates.io Downloads, Lines of Code, Dependencies, Defined Features, AST Nodes, AST Edges, AST Height, Features Nodes, Features Edges, Artifact Nodes, Artifact Edges, Execution Time, Peak Memory Usage")

        for res in results:
            print(format(res))

    except Exception as e:
        printt(f"Error: {e}")

TO_ANALYZE = [
    "https://github.com/GitoxideLabs/gitoxide",
    "https://github.com/denoland/deno",
    "https://github.com/tauri-apps/tauri",
    "https://github.com/rustdesk/rustdesk",
    "https://github.com/FuelLabs/sway",
    "https://github.com/FuelLabs/fuel-core",
    "https://github.com/alacritty/alacritty",
    "https://github.com/rust-lang/rustlings",
    "https://github.com/zed-industries/zed",
    "https://github.com/lencx/ChatGPT",
    "https://github.com/sharkdp/bat",
    "https://github.com/BurntSushi/ripgrep",
    "https://github.com/meilisearch/meilisearch",
    "https://github.com/starship/starship",
    "https://github.com/FuelLabs/fuels-rs",
    "https://github.com/dani-garcia/vaultwarden",
    "https://github.com/bevyengine/bevy",
    "https://github.com/pdm-project/pdm",
    "https://github.com/typst/typst",
    "https://github.com/helix-editor/helix",
    "https://github.com/sharkdp/fd",
    "https://github.com/charliermarsh/ruff",
    "https://github.com/lapce/lapce",
    "https://github.com/tw93/Pake",
    "https://github.com/nushell/nushell",
    "https://github.com/pola-rs/polars",
    "https://github.com/swc-project/swc",
    "https://github.com/influxdata/influxdb",
    "https://github.com/TabbyML/tabby",
    "https://github.com/servo/servo",
    "https://github.com/wasmerio/wasmer",
    "https://github.com/ogham/exa",
    "https://github.com/diem/diem",
    "https://github.com/EmbarkStudios/texture-synthesis",
    "https://github.com/EmbarkStudios/kajiya",
    "https://github.com/EmbarkStudios/rust-gpu",
    "https://github.com/paritytech/substrate",
    "https://github.com/wasmEdge/wasmedge",
    "https://github.com/XAMPPRocky/tokei",
    "https://github.com/quickwit-oss/tantivy",
    "https://github.com/facebook/relay",
    "https://github.com/boa-dev/boa",
    "https://github.com/rerun-io/rerun",
    "https://github.com/containers/podman",
    "https://github.com/hyperium/tonic",
    "https://github.com/tokio-rs/axum",
    "https://github.com/cross-rs/cross",
    "https://github.com/pyroscope-io/pyroscope",
    "https://github.com/bottlerocket-os/bottlerocket"
]

if __name__ == "__main__":
    for repo_url in TO_ANALYZE:
        result = main(repo_url)
