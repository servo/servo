import subprocess
from functools import partial

from mozlog import get_default_logger

logger = None

def vcs(bin_name):
    def inner(command, *args, **kwargs):
        global logger

        if logger is None:
            logger = get_default_logger("vcs")

        repo = kwargs.pop("repo", None)
        log_error = kwargs.pop("log_error", True)
        if kwargs:
            raise TypeError(kwargs)

        args = list(args)

        proc_kwargs = {}
        if repo is not None:
            proc_kwargs["cwd"] = repo

        command_line = [bin_name, command] + args
        logger.debug(" ".join(command_line))
        try:
            return subprocess.check_output(command_line, stderr=subprocess.STDOUT, **proc_kwargs)
        except OSError as e:
            if log_error:
                logger.error(e)
            raise
        except subprocess.CalledProcessError as e:
            if log_error:
                logger.error(e.output)
            raise
    return inner

git = vcs("git")
hg = vcs("hg")


def bind_to_repo(vcs_func, repo, log_error=True):
    return partial(vcs_func, repo=repo, log_error=log_error)


def is_git_root(path, log_error=True):
    try:
        rv = git("rev-parse", "--show-cdup", repo=path, log_error=log_error)
    except subprocess.CalledProcessError:
        return False
    return rv == "\n"
