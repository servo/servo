# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import hashlib
import os
import os.path
import shutil
import stat
import sys
import time
import urllib
import urllib.request
import zipfile
from typing import Dict, List, Union

import six
from io import BufferedIOBase, BytesIO
from socket import error as socket_error

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
SERVO_ROOT = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))


def remove_readonly(func, path, _):
    "Clear the readonly bit and reattempt the removal"
    os.chmod(path, stat.S_IWRITE)
    func(path)


def delete(path):
    if os.path.isdir(path) and not os.path.islink(path):
        shutil.rmtree(path, onerror=remove_readonly)
    else:
        os.remove(path)


def download(description: str, url: str, writer: BufferedIOBase, start_byte: int = 0):
    if start_byte:
        print("Resuming download of {} ...".format(url))
    else:
        print("Downloading {} ...".format(url))
    dumb = (os.environ.get("TERM") == "dumb") or (not sys.stdout.isatty())

    try:
        req = urllib.request.Request(url)
        if start_byte:
            req = urllib.request.Request(url, headers={'Range': 'bytes={}-'.format(start_byte)})
        resp = urllib.request.urlopen(req)

        fsize = None
        if resp.info().get('Content-Length'):
            fsize = int(resp.info().get('Content-Length').strip()) + start_byte

        recved = start_byte
        chunk_size = 64 * 1024

        previous_progress_line = None
        previous_progress_line_time = 0.0
        while True:
            chunk = resp.read(chunk_size)
            if not chunk:
                break
            recved += len(chunk)
            if not dumb:
                if fsize is not None:
                    pct = recved * 100.0 / fsize
                    progress_line = "\rDownloading %s: %5.1f%%" % (description, pct)
                    now = time.time()
                    duration = now - previous_progress_line_time
                    if progress_line != previous_progress_line and duration > .1:
                        print(progress_line, end="")
                        previous_progress_line = progress_line
                        previous_progress_line_time = now

                sys.stdout.flush()
            writer.write(chunk)

        if not dumb:
            print()
    except urllib.error.HTTPError as e:
        print("Download failed ({}): {} - {}".format(e.code, e.reason, url))
        if e.code == 403:
            print("No Rust compiler binary available for this platform. "
                  "Please see https://github.com/servo/servo/#prerequisites")
        sys.exit(1)
    except urllib.error.URLError as e:
        print("Error downloading {}: {}. The failing URL was: {}".format(description, e.reason, url))
        sys.exit(1)
    except socket_error as e:
        print("Looks like there's a connectivity issue, check your Internet connection. {}".format(e))
        sys.exit(1)
    except KeyboardInterrupt:
        writer.flush()
        raise


def download_bytes(description: str, url: str):
    content_writer = BytesIO()
    download(description, url, content_writer)
    return content_writer.getvalue()


def download_file(description: str, url: str, destination_path: str):
    tmp_path = destination_path + ".part"
    try:
        start_byte = os.path.getsize(tmp_path)
        with open(tmp_path, 'ab') as fd:
            download(description, url, fd, start_byte=start_byte)
    except os.error:
        with open(tmp_path, 'wb') as fd:
            download(description, url, fd)
    os.rename(tmp_path, destination_path)


# https://stackoverflow.com/questions/39296101/python-zipfile-removes-execute-permissions-from-binaries
# In particular, we want the executable bit for executable files.
class ZipFileWithUnixPermissions(zipfile.ZipFile):
    def extract(self, member, path=None, pwd=None):
        if not isinstance(member, zipfile.ZipInfo):
            member = self.getinfo(member)

        if path is None:
            path = os.getcwd()

        extracted = self._extract_member(member, path, pwd)
        mode = os.stat(extracted).st_mode
        mode |= (member.external_attr >> 16)
        os.chmod(extracted, mode)
        return extracted

    # For Python 3.x
    def _extract_member(self, member, targetpath, pwd):
        if sys.version_info[0] >= 3:
            if not isinstance(member, zipfile.ZipInfo):
                member = self.getinfo(member)

            targetpath = super()._extract_member(member, targetpath, pwd)

            attr = member.external_attr >> 16
            if attr != 0:
                os.chmod(targetpath, attr)
            return targetpath
        else:
            return super(ZipFileWithUnixPermissions, self)._extract_member(member, targetpath, pwd)


def extract(src, dst, movedir=None, remove=True):
    assert src.endswith(".zip")
    ZipFileWithUnixPermissions(src).extractall(dst)

    if movedir:
        for f in os.listdir(movedir):
            frm = os.path.join(movedir, f)
            to = os.path.join(dst, f)
            os.rename(frm, to)
        os.rmdir(movedir)

    if remove:
        os.remove(src)


def check_hash(filename, expected, algorithm):
    hasher = hashlib.new(algorithm)
    with open(filename, "rb") as f:
        while True:
            block = f.read(16 * 1024)
            if len(block) == 0:
                break
            hasher.update(block)
    if hasher.hexdigest() != expected:
        print("Incorrect {} hash for {}".format(algorithm, filename))
        sys.exit(1)


def get_default_cache_dir(topdir):
    return os.environ.get("SERVO_CACHE_DIR", os.path.join(topdir, ".servo"))


def append_paths_to_env(env: Dict[str, str], key: str, paths: Union[str, List[str]]):
    if isinstance(paths, list):
        paths = os.pathsep.join(paths)

    existing_value = env.get(key, None)
    if existing_value:
        new_value = six.ensure_str(existing_value) + os.pathsep + paths
    else:
        new_value = paths
    env[key] = new_value


def prepend_paths_to_env(env: Dict[str, str], key: str, paths: Union[str, List[str]]):
    if isinstance(paths, list):
        paths = os.pathsep.join(paths)

    existing_value = env.get(key, None)
    new_value = paths
    if existing_value:
        new_value += os.pathsep + six.ensure_str(existing_value)
    env[key] = new_value


def get_target_dir():
    return os.environ.get("CARGO_TARGET_DIR", os.path.join(SERVO_ROOT, "target"))
