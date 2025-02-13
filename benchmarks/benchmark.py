import subprocess
import time
import os
import json
import requests
import toml
import shutil
from datetime import datetime
import signal
import psutil
import io
import sys

# print with timestamp
def printt(message):
    print(f"[{datetime.now().strftime("%Y-%m-%d %H:%M:%S")}] {message}")

# print with timestamp on stderr
def eprintt(message):
    print(f"[{datetime.now().strftime("%Y-%m-%d %H:%M:%S")}] {message}", file=sys.stderr)

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
        eprintt(f"Error fetching GitHub stars: {e}")
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
        eprintt(f"Error fetching crates.io downloads: {e}")
        return None

# return and parse Cargo.toml file
def parse_cargo_toml(subdir="."):
    cargo_toml = os.path.join(subdir, "Cargo.toml")

    if os.path.exists(cargo_toml):
        try:
            with open(cargo_toml, "r") as f:
                return toml.load(f)
        except Exception as e:
            eprintt(f"Error reading Cargo.toml: {e}")

# expand members with wildcard
def expand_members(repo_name, members):
    expanded_members = []
    for member in members:
        if member.endswith("/*"):
            member_dir = os.path.join(repo_name, member[:-2])
            if os.path.exists(member_dir):
                expanded_members.extend([os.path.join(member[:-2], subdir) for subdir in os.listdir(member_dir) if os.path.isdir(os.path.join(member_dir, subdir))])
            else:
                eprintt(f"Warning: Directory {member_dir} does not exist.")
        else:
            expanded_members.append(member)
    return expanded_members

def max_memory():
    current_process = psutil.Process()
    children = current_process.children(recursive=True)
    max_memory = 0
    for child in children:
        try:
            memory_info = child.memory_full_info()
            memory_mb = memory_info.rss / (1024 * 1024)
            max_memory = max(max_memory, memory_mb)
        except psutil.NoSuchProcess as e:
            break
        except psutil.AccessDenied as e:
            pass
        except Exception as e:
            return 0
    return max_memory

# analyze the folder at `repo_name` and return a list of result dicts
def analyze(repo_name, reset_cargo=False):
    eprintt(f"Analyzing {repo_name}.")

    os.chdir(repo_name)

    cargo_toml = parse_cargo_toml(".")
    loc = 0
    for root, _, files in os.walk("."):
        for file in files:
            if file.endswith(".rs"):
                result = subprocess.run(["wc", "-l", os.path.join(root, file)], capture_output=True, text=True)
                loc += int(result.stdout.split()[0])

    result = {
        "error": True,
        "member": repo_name.replace("MEMBER-", ""),
        "lines_of_code": loc,
        "dependencies": len(cargo_toml.get("dependencies", {})),
        "defined_features": len(cargo_toml.get("features", {})),
    }

    if reset_cargo:
        try:
            os.rename("Cargo.toml", "Cargo.toml.bak")
            subprocess.run(["cargo", "init", "--name", "temp"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except Exception as e:
            eprintt(f"Error resetting Cargo.toml: {e}")
            os.chdir("..")
            return result
    subprocess.run(["cargo", "clean"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    start_time = time.time()

    try:
        process = subprocess.Popen(["cargo-rusty-ex", "--print-metadata"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, preexec_fn=os.setsid)
        max_mem = 0
        while not process.poll() and (time.time() - start_time) < 600: # 10 minutes timeout
            max_mem = max(max_mem, max_memory())
            time.sleep(0.01)
        if not process.poll():
            os.killpg(os.getpgid(process.pid), signal.SIGTERM)
            eprintt(f"Error: {repo_name} timed out.")
            os.chdir("..")
            return result
        stdout, stderr = process.communicate()
    except Exception:
        os.killpg(os.getpgid(process.pid), signal.SIGTERM)
        eprintt(f"Error: {repo_name} crashed.")
        os.chdir("..")
        return result

    execution_time = time.time() - start_time

    results = []
    try:
        parsed_line = json.loads(stdout.decode().strip())

        result |= {
            "term_nodes": parsed_line.get("term_nodes", "N/A"),
            "term_edges": parsed_line.get("term_edges", "N/A"),
            "term_height": parsed_line.get("term_height", "N/A"),
            "feature_nodes": parsed_line.get("features_nodes", "N/A"),
            "feature_edges": parsed_line.get("features_edges", "N/A"),
            "feature_squashed_edges": parsed_line.get("features_squashed_edges", "N/A"),
            "artifact_nodes": parsed_line.get("artifacts_nodes", "N/A"),
            "artifact_edges": parsed_line.get("artifacts_edges", "N/A"),
            "execution_time": execution_time,
            "peak_memory_usage": max_mem,
        }

    except json.JSONDecodeError:
        eprintt(f"Error: Line is not a valid JSON.")
        os.chdir("..")
        return result

    os.chdir("..")
    return result | {"error": False}

# launch the analysis for each member of the workspace
def analyze_members(repo_name, members):
    results = []
    for member in members:
        try:
            if not os.path.exists(f"{repo_name}/{member}"):
                eprintt(f"Error: Member {member} does not exist.")
                continue

            base = "MEMBER-" + os.path.basename(member)

            if os.path.exists(base):
                shutil.rmtree(base)

            shutil.copytree(f"{repo_name}/{member}", base)
            results.append(analyze(base, reset_cargo=True))
            shutil.rmtree(base)

        except Exception as e:
            eprintt(f"Error analyzing member {member}: {e}")
            continue
    return results

# format the results as a LaTeX row
def latex_format(results):
    # print("Crate, GH Stars, Crates.io Downloads, Member, Lines of Code, Dependencies, Defined Features, Term Nodes, Term Edges, Term Height, Feature Nodes, Feature Edges, Feature Squashed Edges, Artifact Nodes, Artifact Edges, Execution Time, Peak Memory Usage")

    formatted = []

    for i, res in enumerate(results):
        linebreak = "\\cline{4-17}" if i != len(results) - 1 else "\\hline"

        def g(x):
            r = res.get(x, "N/A")
            return r.replace("_", "\\_") if isinstance(r, str) else r

        if i == 0 and res["error"]:
            formatted.append(f"\\multirow{{{len(results)}}}{{*}}{{\\href{{{g('url')}}}{{{{{g('crate')}}}}}}} & \\multirow{{{len(results)}}}{{*}}{{{g('github_stars')}}} & \\multirow{{{len(results)}}}{{*}}{{{g('cratesio_downloads')}}} & {g('member')} & {g('lines_of_code')} & {g('dependencies')} & {g('defined_features')} & \\multicolumn{{10}}{{c|}}{{\\textit{{error}}}} \\\\ {linebreak}")
            continue

        if i == 0:
            formatted.append(f"\\multirow{{{len(results)}}}{{*}}{{\\href{{{g('url')}}}{{{{{g('crate')}}}}}}} & \\multirow{{{len(results)}}}{{*}}{{{g('github_stars')}}} & \\multirow{{{len(results)}}}{{*}}{{{g('cratesio_downloads')}}} & {g('member')} & {g('lines_of_code')} & {g('dependencies')} & {g('defined_features')} & {g('term_nodes')} & {g('term_edges')} & {g('term_height')} & {g('feature_nodes')} & {g('feature_edges')} & {g('feature_squashed_edges')} & {g('artifact_nodes')} & {g('artifact_edges')} & {g('execution_time'):.2f} s & {int(g('peak_memory_usage'))} MB \\\\ {linebreak}")
            continue

        if res["error"]:
            formatted.append(f"& & & {g('member')} & {g('lines_of_code')} & {g('dependencies')} & {g('defined_features')} & \\multicolumn{{10}}{{c|}}{{\\textit{{error}}}} \\\\ {linebreak}")
            continue

        formatted.append(f"& & & {g('member')} & {g('lines_of_code')} & {g('dependencies')} & {g('defined_features')} & {g('term_nodes')} & {g('term_edges')} & {g('term_height')} & {g('feature_nodes')} & {g('feature_edges')} & {g('feature_squashed_edges')} & {g('artifact_nodes')} & {g('artifact_edges')} & {g('execution_time'):.2f} s & {int(g('peak_memory_usage'))} MB \\\\ {linebreak}")

    return "\n".join(formatted)

def main(repo_url):
    try:
        repo_name = os.path.basename(repo_url).replace(".git", "")
        stars = get_github_stars(repo_url)
        downloads = get_cratesio_downloads(repo_name)

        if os.path.exists(repo_name):
            shutil.rmtree(repo_name)

        eprintt(f"Cloning the repository from {repo_url}.")
        subprocess.run(["git", "clone", "--recurse-submodules", "-j8", repo_url], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        cargo_toml = parse_cargo_toml(repo_name)
        members = cargo_toml.get("workspace", {}).get("members", [])
        members = expand_members(repo_name, members)

        eprintt(f"Members: {members}")

        results = []

        results.append(analyze(repo_name))
        results += analyze_members(repo_name, members)

        results = [r | {"url": repo_url, "crate": repo_name, "github_stars": stars, "cratesio_downloads": downloads} for r in results]

        shutil.rmtree(repo_name)

        with open("results.json", "a") as f:
            json.dump(results, f, indent=4)
            f.write(",\n")

        with open("results.tex", "a") as f:
            f.write(latex_format(results))

    except Exception as e:
        eprintt(f"Error: {e}")

TO_ANALYZE = [
    "https://github.com/sharkdp/bat",
    "https://github.com/GitoxideLabs/gitoxide",
    "https://github.com/FuelLabs/fuels-rs",
    "https://github.com/tauri-apps/tauri",
    "https://github.com/alacritty/alacritty",
    "https://github.com/zed-industries/zed",
    "https://github.com/BurntSushi/ripgrep",
    "https://github.com/meilisearch/meilisearch",
    "https://github.com/rustdesk/rustdesk",
    "https://github.com/typst/typst",
    "https://github.com/helix-editor/helix",
    "https://github.com/charliermarsh/ruff",
    "https://github.com/lapce/lapce",
    "https://github.com/nushell/nushell",
    "https://github.com/pola-rs/polars",
    "https://github.com/swc-project/swc",
    "https://github.com/influxdata/influxdb",
    "https://github.com/TabbyML/tabby",
    "https://github.com/servo/servo",
    "https://github.com/wasmerio/wasmer",
    "https://github.com/diem/diem",
    "https://github.com/EmbarkStudios/texture-synthesis",
    "https://github.com/EmbarkStudios/kajiya",
    "https://github.com/EmbarkStudios/rust-gpu",
    "https://github.com/paritytech/substrate",
    "https://github.com/quickwit-oss/tantivy",
    "https://github.com/hyperium/tonic",
    "https://github.com/n0-computer/sendme",
    "https://github.com/moghtech/komodo",
    "https://github.com/cloudflare/quiche",
    "https://github.com/rolldown/rolldown",
    "https://github.com/n0-computer/iroh",
    "https://github.com/succinctlabs/sp1",
    "https://github.com/unionlabs/union",
    "https://github.com/juspay/hyperswitch",
    "https://github.com/emilk/egui",
    "https://github.com/Nukesor/pueue",
    "https://github.com/denoland/deno",
    "https://github.com/FuelLabs/sway",
    "https://github.com/FuelLabs/fuel-core",
]

if __name__ == "__main__":
    with open("results.json", "a") as f:
        f.write("[\n")

    for repo_url in TO_ANALYZE:
        result = main(repo_url)

    with open("results.json", "a") as f:
        f.write("]\n")
