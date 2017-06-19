import logging
import os
import subprocess
import sys
import tarfile
import zipfile
from io import BytesIO

logger = logging.getLogger(__name__)


class Kwargs(dict):
    def set_if_none(self, name, value, err_fn=None, desc=None, extra_cond=None):
        if desc is None:
            desc = name

        if self[name] is None:
            if extra_cond is not None and not extra_cond(self):
                return
            if callable(value):
                value = value()
            if not value:
                if err_fn is not None:
                    return err_fn(kwargs, "Failed to find %s" % desc)
                else:
                    return
            self[name] = value
            logger.info("Set %s to %s" % (desc, value))


def call(*args):
    """Log terminal command, invoke it as a subprocess.

    Returns a bytestring of the subprocess output if no error.
    """
    logger.debug("%s" % " ".join(args))
    try:
        return subprocess.check_output(args)
    except subprocess.CalledProcessError as e:
        logger.critical("%s exited with return code %i" %
                        (e.cmd, e.returncode))
        logger.critical(e.output)
        raise


def get_git_cmd(repo_path):
    """Create a function for invoking git commands as a subprocess."""
    def git(cmd, *args):
        full_cmd = ["git", cmd] + list(args)
        try:
            logger.debug(" ".join(full_cmd))
            return subprocess.check_output(full_cmd, cwd=repo_path, stderr=subprocess.STDOUT).strip()
        except subprocess.CalledProcessError as e:
            logger.error("Git command exited with status %i" % e.returncode)
            logger.error(e.output)
            sys.exit(1)
    return git


def seekable(fileobj):
    """Attempt to use file.seek on given file, with fallbacks."""
    try:
        fileobj.seek(fileobj.tell())
    except Exception:
        return BytesIO(fileobj.read())
    else:
        return fileobj


def untar(fileobj, dest="."):
    """Extract tar archive."""
    logger.debug("untar")
    fileobj = seekable(fileobj)
    with tarfile.open(fileobj=fileobj) as tar_data:
        tar_data.extractall(path=dest)


def unzip(fileobj, dest=None, limit=None):
    """Extract zip archive."""
    logger.debug("unzip")
    fileobj = seekable(fileobj)
    with zipfile.ZipFile(fileobj) as zip_data:
        for info in zip_data.infolist():
            if limit is not None and info.filename not in limit:
                continue
            zip_data.extract(info, path=dest)
            perm = info.external_attr >> 16 & 0x1FF
            os.chmod(os.path.join(dest, info.filename), perm)


class pwd(object):
    """Create context for temporarily changing present working directory."""
    def __init__(self, dir):
        self.dir = dir
        self.old_dir = None

    def __enter__(self):
        self.old_dir = os.path.abspath(os.curdir)
        os.chdir(self.dir)

    def __exit__(self, *args, **kwargs):
        os.chdir(self.old_dir)
        self.old_dir = None


def get(url):
    """Issue GET request to a given URL and return the response."""
    import requests

    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    return resp
