import subprocess
import sys
from functools import partial

def vcs(bin_name):
    def inner(command, *args, **kwargs):
        repo = kwargs.pop("repo", None)
        log_error = kwargs.pop("log_error", True)
        quiet = kwargs.pop("quiet", False)
        if kwargs:
            raise TypeError, kwargs

        args = list(args)

        proc_kwargs = {}
        if repo is not None:
            proc_kwargs["cwd"] = repo

        command_line = [bin_name, command] + args
        if not quiet:
            print >> sys.stderr, " ".join(command_line[:10])
        try:
            return subprocess.check_output(command_line, stderr=subprocess.STDOUT, **proc_kwargs)
        except subprocess.CalledProcessError as e:
            if log_error:
                print >> sys.stderr, e.output
            raise
    return inner

git = vcs("git")
hg = vcs("hg")


def bind_to_repo(vcs_func, repo):
    return partial(vcs_func, repo=repo)


def is_git_root(path):
    try:
        rv = git("rev-parse", "--show-cdup", repo=path)
    except subprocess.CalledProcessError:
        return False
    return rv == "\n"
