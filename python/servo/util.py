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
import os.path as path
import sys
from socket import error as socket_error
import StringIO
import tarfile
import zipfile
import urllib2


def download(desc, src, writer, start_byte=0):
    if start_byte:
        print("Resuming download of %s..." % desc)
    else:
        print("Downloading %s..." % desc)
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
        print("Download failed (%d): %s - %s" % (e.code, e.reason, src))
        if e.code == 403:
            print("No Rust compiler binary available for this platform. "
                  "Please see https://github.com/servo/servo/#prerequisites")
        sys.exit(1)
    except urllib2.URLError, e:
        print("Error downloading Rust compiler: %s. The failing URL was: %s" % (e.reason, src))
        sys.exit(1)
    except socket_error, e:
        print("Looks like there's a connectivity issue, check your Internet connection. %s" % (e))
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
        start_byte = path.getsize(tmp_path)
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
            frm = path.join(movedir, f)
            to = path.join(dst, f)
            os.rename(frm, to)
        os.rmdir(movedir)

    os.remove(src)
