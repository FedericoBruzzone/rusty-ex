import subprocess
import time
import os
import json
import sys
import requests
import toml

def get_github_stars(repo_url):
    try:
        repo_path = repo_url.split("github.com/")[-1].replace(".git", "")
        api_url = f"https://api.github.com/repos/{repo_path}"
        response = requests.get(api_url)
        response.raise_for_status()
        data = response.json()
        return data.get("stargazers_count", 0)
    except requests.exceptions.RequestException as e:
        print(f"Error fetching GitHub stars: {e}")
        return None
    except Exception as e:
        print(f"Unexpected error: {e}")
        return None

def count_cargo_toml_data(repo_name):
    cargo_toml_path = os.path.join(repo_name, "Cargo.toml")
    workspace_members = 0
    feature_count = 0

    if os.path.exists(cargo_toml_path):
        try:
            with open(cargo_toml_path, "r") as f:
                cargo_data = toml.load(f)

                if "workspace" in cargo_data and "members" in cargo_data["workspace"]:
                    workspace_members = cargo_data["workspace"]["members"]

                if "features" in cargo_data:
                    feature_count = len(cargo_data["features"])
        except Exception as e:
            print(f"Error reading Cargo.toml: {e}")

    return workspace_members, feature_count

def monitor_process(repo_url, output_file="output.txt"):
    try:
        repo_name = os.path.basename(repo_url).replace('.git', '')
        print(f"Analyzing {repo_name}.")

        stars = get_github_stars(repo_url)

        if not os.path.exists(repo_name):
            print(f"Cloning the repository from {repo_url}.")
            subprocess.run(["git", "clone", repo_url], check=True)
        else:
            print(f"Repository {repo_name} already exists, skipping git clone.")

        os.chdir(repo_name)
        subprocess.run(["cargo", "clean"], check=True)

        workspace_members, feature_count = count_cargo_toml_data(".")

        start_time = time.time()
        process = subprocess.Popen(["cargo-rustc-ex", "--print-metadata"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

        stdout, stderr = process.communicate()
        end_time = time.time()
        execution_time = end_time - start_time

        stdout_decoded = stdout.decode()
        with open(output_file, "w") as f:
            f.write(stdout_decoded)

        latex_rows = []
        for line in stdout_decoded.splitlines():
            if not line.strip():
                continue
            try:
                parsed_line = json.loads(line)
                ast_nodes = parsed_line.get("ast_nodes", "N/A")
                ast_height = parsed_line.get("ast_height", "N/A")
                features_nodes = parsed_line.get("features_nodes", "N/A")
                artifacts_nodes = parsed_line.get("artifacts_nodes", "N/A")

                latex_row = f"\\href{{{repo_url}}}{{\\underline{{{repo_name}}}}} & {stars} & {len(workspace_members)} & {feature_count} & {ast_nodes} & {ast_height} & {features_nodes} & {artifacts_nodes} & {execution_time:.2f} s \\\\ \\hline"

                latex_rows.append(latex_row)
            except json.JSONDecodeError:
                print(f"Error: Line is not a valid JSON.")
                continue

        if not latex_rows:
            print("Error: Output is empty or no valid JSON found.")
            return

        latex_table = "\n".join(latex_rows)
        print(latex_table)

    except FileNotFoundError:
        print(f"Error: Repository or binary file not found.")
        return None
    except Exception as e:
        print(f"Error: {e}")
        return None

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python monitor.py <repository_url>")
        sys.exit(1)

    repo_url = sys.argv[1]
    result = monitor_process(repo_url)
