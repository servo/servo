import errno
import logging
import os
import shutil
import stat
import subprocess
import tarfile
import time
import zipfile
from io import BytesIO
from socket import error as SocketError  # NOQA: N812
from urllib.request import urlopen

logger = logging.getLogger(__name__)


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


def get_download_to_descriptor(fd, url, max_retries=5):
    """Download an URL in chunks and saves it to a file descriptor (truncating it)
    It doesn't close the descriptor, but flushes it on success.
    It retries the download in case of ECONNRESET up to max_retries.
    This function is meant to download big files directly to the disk without
    caching the whole file in memory.
    """
    if max_retries < 1:
        max_retries = 1
    wait = 2
    for current_retry in range(1, max_retries+1):
        try:
            logger.info("Downloading %s Try %d/%d" % (url, current_retry, max_retries))
            resp = urlopen(url)
            # We may come here in a retry, ensure to truncate fd before start writing.
            fd.seek(0)
            fd.truncate(0)
            while True:
                chunk = resp.read(16*1024)
                if not chunk:
                    break  # Download finished
                fd.write(chunk)
            fd.flush()
            # Success
            return
        except SocketError as e:
            if current_retry < max_retries and e.errno == errno.ECONNRESET:
                # Retry
                logger.error("Connection reset by peer. Retrying after %ds..." % wait)
                time.sleep(wait)
                wait *= 2
            else:
                # Maximum retries or unknown error
                raise

def rmtree(path):
    # This works around two issues:
    # 1. Cannot delete read-only files owned by us (e.g. files extracted from tarballs)
    # 2. On Windows, we sometimes just need to retry in case the file handler
    #    hasn't been fully released (a common issue).
    def handle_remove_readonly(func, path, exc):
        excvalue = exc[1]
        if func in (os.rmdir, os.remove, os.unlink) and excvalue.errno == errno.EACCES:
            os.chmod(path, stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)  # 0777
            func(path)
        else:
            raise

    return shutil.rmtree(path, onerror=handle_remove_readonly)


def sha256sum(file_path):
    """Computes the SHA256 hash sum of a file"""
    from hashlib import sha256
    hash = sha256()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            hash.update(chunk)
    return hash.hexdigest()
