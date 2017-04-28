import itertools
import os
import shutil
import subprocess
import sys

import vcs

here = os.path.abspath(os.path.dirname(__file__))

source_dir = os.path.join(here, "..", "..")

remote_built = "https://github.com/jgraham/css-test-built.git"
built_dir = os.path.join(here, "css-test-built")

local_files = ["manifest", "serve", "serve.py", ".gitmodules", "tools", "resources",
               "config.default.json"]

def get_hgsubstate():
    state = {}
    with open(os.path.join(source_dir, ".hgsubstate"), "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            revision, path = line.split(" ", 1)
            state[path] = revision
    return state

def fetch_submodules():
    hg = vcs.hg
    orig_dir = os.getcwd()
    state = get_hgsubstate()
    for tool in ["apiclient", "w3ctestlib"]:
        dest_dir = os.path.join(source_dir, "tools", tool)
        repo_path = "tools/" + tool
        if os.path.exists(os.path.join(dest_dir, ".hg")):
            try:
                os.chdir(dest_dir)
                if repo_path in state:
                    rev = state[repo_path]
                    try:
                        hg("update", rev, log_error=False)
                    except subprocess.CalledProcessError:
                        hg("pull")
                        hg("update", rev)
                else:
                    hg("pull")
                    hg("update")
            finally:
                os.chdir(orig_dir)
        else:
            hg("clone", ("https://hg.csswg.org/dev/%s" % tool), dest_dir)
            try:
                os.chdir(dest_dir)
                if repo_path in state:
                    hg("update", state[repo_path])
                else:
                    hg("update")
            finally:
                os.chdir(orig_dir)

def update_dist():
    if not os.path.exists(built_dir) or not vcs.is_git_root(built_dir):
        git = vcs.git
        git("clone", "--depth", "1", remote_built, built_dir)
    else:
        git = vcs.bind_to_repo(vcs.git, built_dir)
        git("fetch")
        if "origin/master" in git("branch", "-a"):
            git("checkout", "master")
            git("merge", "--ff-only", "origin/master")

    git = vcs.bind_to_repo(vcs.git, built_dir)
    git("config", "user.email", "CssBuildBot@users.noreply.github.com")
    git("config", "user.name", "CSS Build Bot")
    git("submodule", "update", "--init", "--recursive")

def setup_virtualenv():
    virtualenv_path = os.path.join(here, "_virtualenv")

    if not os.path.exists(virtualenv_path):
        subprocess.check_call(["virtualenv", virtualenv_path])

    activate_path = os.path.join(virtualenv_path, "bin", "activate_this.py")

    execfile(activate_path, dict(__file__=activate_path))

    subprocess.check_call(["pip", "-q", "install", "mercurial"])
    subprocess.check_call(["pip", "-q", "install", "html5lib==0.9999999"])
    subprocess.check_call(["pip", "-q", "install", "lxml"])
    subprocess.check_call(["pip", "-q", "install", "Template-Python"])


def update_to_changeset(changeset):
    git = vcs.bind_to_repo(vcs.git, source_dir)
    git("checkout", changeset)
    apply_build_system_fixes()

def apply_build_system_fixes():
    fixes = [
        "c017547f65e07bdd889736524d47824d032ba2e8",
        "cb4a737a88aa7e2f4e54383c57ffa2dfae093dcf",
        "ec540343a3e729644c8178dbcf6d063dca20d49f",
    ]
    git = vcs.bind_to_repo(vcs.git, source_dir)
    for fix in fixes:
        git("cherry-pick", "--keep-redundant-commits", fix)

def build_tests():
    subprocess.check_call(["python", os.path.join(source_dir, "tools", "build.py")],
                           cwd=source_dir)

def remove_current_files():
    for node in os.listdir(built_dir):
        if node.startswith(".git"):
            continue

        if node in ("resources", "tools"):
            continue

        path = os.path.join(built_dir, node)
        if os.path.isdir(path):
            shutil.rmtree(path)
        else:
            os.remove(path)

def copy_files():
    dist_path = os.path.join(source_dir, "dist")
    for node in os.listdir(dist_path):
        src_path = os.path.join(dist_path, node)
        dest_path = os.path.join(built_dir, node)
        if os.path.isdir(src_path):
            shutil.copytree(src_path, dest_path)
        else:
            shutil.copy2(src_path, dest_path)

def update_git():
    git = vcs.bind_to_repo(vcs.git, built_dir)
    git("add", ".")

def add_changeset(changeset):
    git = vcs.bind_to_repo(vcs.git, built_dir)

    dest_path = os.path.join(built_dir, "source_rev")
    with open(dest_path, "w") as f:
        f.write(changeset)
    git("add", os.path.relpath(dest_path, built_dir))

def commit(changeset):
    git = vcs.git

    msg = git("log", "-r", changeset, "-n", "1", "--pretty=%B", repo=source_dir)
    msg = "%s\n\nBuild from revision %s" % (msg, changeset)

    git("commit", "-m", msg, repo=built_dir)

def get_new_commits():
    git = vcs.bind_to_repo(vcs.git, source_dir)
    commit_path = os.path.join(built_dir, "source_rev")
    with open(commit_path) as f:
        prev_commit = f.read().strip()

    if git("rev-parse", "--revs-only", prev_commit).strip() != prev_commit:
        # we don't have prev_commit in current tree, so let's just do what's new
        commit_range = os.environ['TRAVIS_COMMIT_RANGE']
        assert (os.environ["TRAVIS_PULL_REQUEST"] != "false" or
                os.environ["TRAVIS_BRANCH"] != "master")
    else:
        merge_base = git("merge-base", prev_commit, os.environ['TRAVIS_COMMIT']).strip()
        commit_range = "%s..%s" % (merge_base, os.environ['TRAVIS_COMMIT'])

    commits = git("log", "--pretty=%H", "-r", commit_range).strip()
    if not commits:
        return []
    return reversed(commits.split("\n"))

def maybe_push():
    if os.environ["TRAVIS_PULL_REQUEST"] != "false":
        return

    if os.environ["TRAVIS_BRANCH"] != "master":
        return

    git = vcs.bind_to_repo(vcs.git, built_dir)

    out = "https://%s@github.com/jgraham/css-test-built.git" % os.environ["TOKEN"]
    git("remote", "add", "out", out, quiet=True)

    for i in range(2):
        try:
            git("push", "out", "HEAD:master")
        except subprocess.CalledProcessError:
            if i == 0:
                git("fetch", "origin")
                git("rebase", "origin/master")
        else:
            return

    raise Exception("Push failed")

def main():
    setup_virtualenv()
    fetch_submodules()
    update_dist()
    changesets = list(get_new_commits())
    print >> sys.stderr, "Building %d changesets:" % len(changesets)
    print >> sys.stderr, "\n".join(changesets)
    if len(changesets) > 50:
        raise Exception("Building more than 50 changesets, giving up")

    for changeset in changesets:
        update_to_changeset(changeset)
        remove_current_files()
        build_tests()
        copy_files()
        update_git()
        add_changeset(changeset)
        commit(changeset)
    maybe_push()

if __name__ == "__main__":
    main()
