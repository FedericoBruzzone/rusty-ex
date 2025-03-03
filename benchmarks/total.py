# Create the last row of the latex table for the aggregated projects.
# This is the total of each column.

import json

def read_json_file(file_path):
    with open(file_path, "r") as f:
        data = json.load(f)
    return data

def repo_total(members):

    root = members[0]

    result = {
        "members": len(members),
        "errors": 0,
        "lines_of_code": root["lines_of_code"],
        "dependencies": 0,
        "defined_features": 0,
        "term_nodes": 0,
        "term_edges": 0,
        "term_height": 0,
        "feature_nodes": 0,
        "feature_edges": 0,
        "feature_squashed_edges": 0,
        "artifact_nodes": 0,
        "artifact_edges": 0,
        "execution_time": 0,
        "peak_memory_usage": 0,
        "url": root["url"],
        "crate": root["crate"],
        "github_stars": root["github_stars"],
        "cratesio_downloads": root["cratesio_downloads"]
    }

    for m in members:
        if m["error"]:
            result["errors"] += 1
            continue
        result["dependencies"] += m["dependencies"]
        result["defined_features"] += m["defined_features"]
        result["term_nodes"] += m["term_nodes"]
        result["term_edges"] += m["term_edges"]
        result["term_height"] = max(result["term_height"], m["term_height"])
        result["feature_nodes"] += m["feature_nodes"]
        result["feature_edges"] += m["feature_edges"]
        result["feature_squashed_edges"] += m["feature_squashed_edges"]
        result["artifact_nodes"] += m["artifact_nodes"]
        result["artifact_edges"] += m["artifact_edges"]
        result["execution_time"] += m["execution_time"]
        result["peak_memory_usage"] = max(result["peak_memory_usage"], m["peak_memory_usage"])
    return result

# format the results as a LaTeX row
def latex_format(res):
    def g(x):
        r = res.get(x, "N/A")
        return r.replace("_", "\\_") if isinstance(r, str) else r

    return f"\\href{{{g('url')}}}{{{{{g('crate')}}}}} & {g('github_stars')} & {g('cratesio_downloads')} & {g('lines_of_code')} & {g('members')} & {g('errors')} & {g('dependencies')} & {g('defined_features')} & {g('term_nodes')} & {g('term_edges')} & {g('term_height')} & {g('feature_nodes')} & {g('feature_edges')} & {g('feature_squashed_edges')} & {g('artifact_nodes')} & {g('artifact_edges')} & {g('execution_time'):.2f} s & {int(g('peak_memory_usage'))} MB \\\\ \\hline"

if __name__ == "__main__":
    file_path = "results.json"
    data = read_json_file(file_path)
    for repo in data:
        result = repo_total(repo)
        # print(result)
        print(latex_format(result))
