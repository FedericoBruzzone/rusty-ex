# Create the latex table for the aggregated projects.
# The latest row (the total) can be created by calling the total.py script.

import json

# format a single project composed of multiple crates as a LaTeX table of multiple rows
def latex_crates(results):
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

# format a single aggregated project as a LaTeX table row
def latex_project(result):
    def g(x):
        r = result.get(x, "N/A")
        return r.replace("_", "\\_") if isinstance(r, str) else int(r)

    return f"\\href{{{g('url')}}}{{{{{g('crate')}}}}} & {g('github_stars')} & {g('cratesio_downloads')} & {g('lines_of_code')} & {g('members')} & {g('errors')} & {g('dependencies')} & {g('defined_features')} & {g('term_nodes')} & {g('term_edges')} & {g('term_height')} & {g('feature_nodes')} & {g('feature_edges')} & {g('feature_squashed_edges')} & {g('artifact_nodes')} & {g('artifact_edges')} & {g('execution_time')} s & {g('peak_memory_usage')} MB \\\\ \\hline"


def main():
    # format single crates
    with open("data/analyzed-crates.json") as f:
        crates = json.load(f)

    with open("crates.tex", "a") as f:
        buffer = []
        for c in crates:
            if not buffer or c["crate"] == buffer[0]["crate"]:
                buffer.append(c)
            else:
                f.write(latex_crates(buffer) + "\n")
                buffer = [c]
        if buffer:
            f.write(latex_crates(buffer) + "\n")

    # format aggregated projects
    with open("data/analyzed-projects.json") as f:
        projects = json.load(f)

    with open("projects.tex", "a") as f:
        for p in projects:
            f.write(latex_project(p) + "\n")

if __name__ == "__main__":
    main()
