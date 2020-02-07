# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import absolute_import, print_function, unicode_literals

import hashlib
import os
import os.path
import platform
import shutil
from socket import error as socket_error
import stat
from io import BytesIO
import sys
import time
import zipfile
import six.moves.urllib as urllib


try:
    from ssl import HAS_SNI
except ImportError:
    HAS_SNI = False

HAS_SNI_AND_RECENT_PYTHON = HAS_SNI and sys.version_info >= (2, 7, 9)


def get_static_rust_lang_org_dist():
    if HAS_SNI_AND_RECENT_PYTHON:
        return "https://static.rust-lang.org/dist"

    return "https://static-rust-lang-org.s3.amazonaws.com/dist"


def get_urlopen_kwargs():
    # The cafile parameter was added in 2.7.9
    if HAS_SNI_AND_RECENT_PYTHON:
        import certifi
        return {"cafile": certifi.where()}
    return {}


def remove_readonly(func, path, _):
    "Clear the readonly bit and reattempt the removal"
    os.chmod(path, stat.S_IWRITE)
    func(path)


def delete(path):
    if os.path.isdir(path) and not os.path.islink(path):
        shutil.rmtree(path, onerror=remove_readonly)
    else:
        os.remove(path)


def host_platform():
    os_type = platform.system().lower()
    if os_type == "linux":
        os_type = "unknown-linux-gnu"
    elif os_type == "darwin":
        os_type = "apple-darwin"
    elif os_type == "android":
        os_type = "linux-androideabi"
    elif os_type == "windows":
        os_type = "pc-windows-msvc"
    elif os_type == "freebsd":
        os_type = "unknown-freebsd"
    else:
        os_type = "unknown"
    return os_type


def host_triple():
    os_type = host_platform()
    cpu_type = platform.machine().lower()
    if cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
        cpu_type = "i686"
    elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
        cpu_type = "x86_64"
    elif cpu_type == "arm":
        cpu_type = "arm"
    elif cpu_type == "aarch64":
        cpu_type = "aarch64"
    else:
        cpu_type = "unknown"

    return "{}-{}".format(cpu_type, os_type)


def download(desc, src, writer, start_byte=0):
    if start_byte:
        print("Resuming download of {} ...".format(src))
    else:
        print("Downloading {} ...".format(src))
    dumb = (os.environ.get("TERM") == "dumb") or (not sys.stdout.isatty())

    try:
        req = urllib.request.Request(src)
        if start_byte:
            req = urllib.request.Request(src, headers={'Range': 'bytes={}-'.format(start_byte)})
        resp = urllib.request.urlopen(req, **get_urlopen_kwargs())

        fsize = None
        if resp.info().get('Content-Length'):
            fsize = int(resp.info().get('Content-Length').strip()) + start_byte

        recved = start_byte
        chunk_size = 64 * 1024

        previous_progress_line = None
        previous_progress_line_time = 0
        while True:
            chunk = resp.read(chunk_size)
            if not chunk:
                break
            recved += len(chunk)
            if not dumb:
                if fsize is not None:
                    pct = recved * 100.0 / fsize
                    progress_line = "\rDownloading %s: %5.1f%%" % (desc, pct)
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
        print("Download failed ({}): {} - {}".format(e.code, e.reason, src))
        if e.code == 403:
            print("No Rust compiler binary available for this platform. "
                  "Please see https://github.com/servo/servo/#prerequisites")
        sys.exit(1)
    except urllib.error.URLError as e:
        print("Error downloading {}: {}. The failing URL was: {}".format(desc, e.reason, src))
        sys.exit(1)
    except socket_error as e:
        print("Looks like there's a connectivity issue, check your Internet connection. {}".format(e))
        sys.exit(1)
    except KeyboardInterrupt:
        writer.flush()
        raise


def download_bytes(desc, src):
    content_writer = BytesIO()
    download(desc, src, content_writer)
    return content_writer.getvalue()


def download_file(desc, src, dst):
    tmp_path = dst + ".part"
    try:
        start_byte = os.path.getsize(tmp_path)
        with open(tmp_path, 'ab') as fd:
            download(desc, src, fd, start_byte=start_byte)
    except os.error:
        with open(tmp_path, 'wb') as fd:
            download(desc, src, fd)
    os.rename(tmp_path, dst)


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
