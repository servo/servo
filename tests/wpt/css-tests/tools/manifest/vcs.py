import os
import subprocess

def get_git_func(repo_path):
    def git(cmd, *args):
        full_cmd = ["git", cmd] + list(args)
        return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT)
    return git


def is_git_repo(tests_root):
    return os.path.exists(os.path.join(tests_root, ".git"))


_repo_root = None
def get_repo_root(initial_dir=None):
    global _repo_root

    if initial_dir is None:
        initial_dir = os.path.dirname(__file__)

    if _repo_root is None:
        git = get_git_func(initial_dir)
        _repo_root = git("rev-parse", "--show-toplevel").rstrip()
    return _repo_root
