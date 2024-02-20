import json
import os
import subprocess
import sys

HERE = os.path.dirname(__file__)
DEST = os.path.abspath(os.path.join(HERE, "../extern"))


def download(repo: str, branch_or_tag: str):
    subprocess.check_call(
        ["git", "clone", "--depth=1", "-b", branch_or_tag, repo], cwd=DEST
    )


def main(argv):
    deps_file_path = argv[0] if argv else os.path.join(DEST, "deps.json")
    with open(deps_file_path) as file:
        deps = json.load(file)
    for dep in deps:
        download(dep["repo"], dep.get("branch") or dep["tag"])


if __name__ == "__main__":
    main(sys.argv[1:])
