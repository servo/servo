# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import absolute_import, print_function, unicode_literals

import os
import os.path
import platform
import shutil
from socket import error as socket_error
import StringIO
import sys
import tarfile
import zipfile
import urllib2


def delete(path):
    if os.path.isdir(path) and not os.path.islink(path):
        shutil.rmtree(path)
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
        # If we are in a Visual Studio environment, use msvc
        if os.getenv("PLATFORM") is not None:
            os_type = "pc-windows-msvc"
        else:
            os_type = "unknown"
    elif os_type == "freebsd":
        os_type = "unknown-freebsd"
    else:
        os_type = "unknown"
    return os_type


def host_triple():
    os_type = host_platform()
    cpu_type = platform.machine().lower()
    if os_type.endswith("-msvc"):
        # vcvars*.bat should set it properly
        platform_env = os.environ.get("PLATFORM").upper()
        if platform_env == "X86":
            cpu_type = "i686"
        elif platform_env == "X64":
            cpu_type = "x86_64"
        else:
            cpu_type = "unknown"
    elif cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
        cpu_type = "i686"
    elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
        cpu_type = "x86_64"
    elif cpu_type == "arm":
        cpu_type = "arm"
    else:
        cpu_type = "unknown"

    return "{}-{}".format(cpu_type, os_type)


def download(desc, src, writer, start_byte=0):
    if start_byte:
        print("Resuming download of {}...".format(desc))
    else:
        print("Downloading {}...".format(desc))
    dumb = (os.environ.get("TERM") == "dumb") or (not sys.stdout.isatty())

    try:
        req = urllib2.Request(src)
        if start_byte:
            req = urllib2.Request(src, headers={'Range': 'bytes={}-'.format(start_byte)})
        resp = urllib2.urlopen(req)

        fsize = None
        if resp.info().getheader('Content-Length'):
            fsize = int(resp.info().getheader('Content-Length').strip()) + start_byte

        recved = start_byte
        chunk_size = 8192

        while True:
            chunk = resp.read(chunk_size)
            if not chunk:
                break
            recved += len(chunk)
            if not dumb:
                if fsize is not None:
                    pct = recved * 100.0 / fsize
                    print("\rDownloading %s: %5.1f%%" % (desc, pct), end="")

                sys.stdout.flush()
            writer.write(chunk)

        if not dumb:
            print()
    except urllib2.HTTPError, e:
        print("Download failed ({}): {} - {}".format(e.code, e.reason, src))
        if e.code == 403:
            print("No Rust compiler binary available for this platform. "
                  "Please see https://github.com/servo/servo/#prerequisites")
        sys.exit(1)
    except urllib2.URLError, e:
        print("Error downloading {}: {}. The failing URL was: {}".format(desc, e.reason, src))
        sys.exit(1)
    except socket_error, e:
        print("Looks like there's a connectivity issue, check your Internet connection. {}".format(e))
        sys.exit(1)
    except KeyboardInterrupt:
        writer.flush()
        raise


def download_bytes(desc, src):
    content_writer = StringIO.StringIO()
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


def extract(src, dst, movedir=None):
    if src.endswith(".zip"):
        zipfile.ZipFile(src).extractall(dst)
    else:
        tarfile.open(src).extractall(dst)

    if movedir:
        for f in os.listdir(movedir):
            frm = os.path.join(movedir, f)
            to = os.path.join(dst, f)
            os.rename(frm, to)
        os.rmdir(movedir)

    os.remove(src)
