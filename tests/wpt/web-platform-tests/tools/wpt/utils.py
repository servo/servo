import logging
import os
import subprocess
import tarfile
import zipfile
from io import BytesIO

try:
    from typing import Any, Callable
except ImportError:
    pass

logger = logging.getLogger(__name__)


class Kwargs(dict):
    def set_if_none(self,
                    name,            # type: str
                    value,           # type: Any
                    err_fn=None,     # type: Callable[[Kwargs, str], Any]
                    desc=None,       # type: str
                    extra_cond=None  # type: Callable[[Kwargs], Any]
                    ):
        # type: (...) -> Any
        if desc is None:
            desc = name

        if name not in self or self[name] is None:
            if extra_cond is not None and not extra_cond(self):
                return
            if callable(value):
                value = value()
            if not value:
                if err_fn is not None:
                    return err_fn(self, "Failed to find %s" % desc)
                else:
                    return
            self[name] = value
            logger.info("Set %s to %s" % (desc, value))


def call(*args):
    """Log terminal command, invoke it as a subprocess.

    Returns a bytestring of the subprocess output if no error.
    """
    logger.debug(" ".join(args))
    try:
        return subprocess.check_output(args).decode('utf8')
    except subprocess.CalledProcessError as e:
        logger.critical("%s exited with return code %i" %
                        (e.cmd, e.returncode))
        logger.critical(e.output)
        raise


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


def get(url):
    """Issue GET request to a given URL and return the response."""
    import requests

    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    return resp
