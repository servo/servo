# mypy: allow-untyped-defs

import errno
import logging
import os
import shutil
import stat
import subprocess
import sys
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
            # external_attr has a size of 4 bytes and the info it contains depends on the system where the ZIP file was created.
            # - If the Zipfile was created on an UNIX environment, then the 2 highest bytes represent UNIX permissions and file
            #   type bits (sys/stat.h st_mode entry on struct stat) and the lowest byte represents DOS FAT compatibility attributes
            #   (used mainly to store the directory bit).
            # - If the ZipFile was created on a WIN/DOS environment then the lowest byte represents DOS FAT file attributes
            #   (those attributes are: directory bit, hidden bit, read-only bit, system-file bit, etc).
            # More info at https://unix.stackexchange.com/a/14727 and https://forensicswiki.xyz/page/ZIP
            # So, we can ignore the DOS FAT attributes because python ZipFile.extract() already takes care of creating the directories
            # as needed (both on win and *nix) and the other DOS FAT attributes (hidden/read-only/system-file/etc) are not interesting
            # here (not even on Windows, since we don't care about setting those extra attributes for our use case).
            # So we do this:
            #   1. When uncompressing on a Windows system we just call to extract().
            #   2. When uncompressing on an Unix-like system we only take care of the attributes if the zip file was created on an
            #      Unix-like system, otherwise we don't have any info about the file permissions other than the DOS FAT attributes,
            #      which are useless here, so just call to extract() without setting any specific file permission in that case.
            if info.create_system == 0 or sys.platform == 'win32':
                zip_data.extract(info, path=dest)
            else:
                stat_st_mode = info.external_attr >> 16
                info_dst_path = os.path.join(dest, info.filename)
                if stat.S_ISLNK(stat_st_mode):
                    # Symlinks are stored in the ZIP file as text files that contain inside the target filename of the symlink.
                    # Recreate the symlink instead of calling extract() when an entry with the attribute stat.S_IFLNK is detected.
                    link_src_path = zip_data.read(info)
                    link_dst_dir = os.path.dirname(info_dst_path)
                    if not os.path.isdir(link_dst_dir):
                        os.makedirs(link_dst_dir)

                    # Remove existing link if exists.
                    if os.path.islink(info_dst_path):
                        os.unlink(info_dst_path)
                    os.symlink(link_src_path, info_dst_path)
                else:
                    zip_data.extract(info, path=dest)
                    # Preserve bits 0-8 only: rwxrwxrwx (no sticky/setuid/setgid bits).
                    perm = stat_st_mode & 0x1FF
                    os.chmod(info_dst_path, perm)


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


# see https://docs.python.org/3/whatsnew/3.12.html#imp
def load_source(modname, filename):
    import importlib.machinery
    import importlib.util

    loader = importlib.machinery.SourceFileLoader(modname, filename)
    spec = importlib.util.spec_from_file_location(modname, filename, loader=loader)
    module = importlib.util.module_from_spec(spec)
    sys.modules[module.__name__] = module
    loader.exec_module(module)
    return module
