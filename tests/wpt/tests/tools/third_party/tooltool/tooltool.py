#!/usr/bin/env python3

# tooltool is a lookaside cache implemented in Python
# Copyright (C) 2011 John H. Ford <john@johnford.info>
#
# This program is free software; you can redistribute it and/or
# modify it under the terms of the GNU General Public License
# as published by the Free Software Foundation version 2
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
# 02110-1301, USA.

# A manifest file specifies files in that directory that are stored
# elsewhere. This file should only list files in the same directory
# in which the manifest file resides and it should be called
# 'manifest.tt'

import base64
import calendar
import hashlib
import hmac
import json
import logging
import math
import optparse
import os
import pprint
import re
import shutil
import stat
import sys
import tarfile
import tempfile
import threading
import time
import zipfile
from contextlib import closing, contextmanager
from functools import wraps
from io import BytesIO, open
from random import random
from subprocess import PIPE, Popen

__version__ = "1.4.0"

# Allowed request header characters:
# !#$%&'()*+,-./:;<=>?@[]^_`{|}~ and space, a-z, A-Z, 0-9, \, "
REQUEST_HEADER_ATTRIBUTE_CHARS = re.compile(
    r"^[ a-zA-Z0-9_\!#\$%&'\(\)\*\+,\-\./\:;<\=>\?@\[\]\^`\{\|\}~]*$"
)
DEFAULT_MANIFEST_NAME = "manifest.tt"
TOOLTOOL_PACKAGE_SUFFIX = ".TOOLTOOL-PACKAGE"
HAWK_VER = 1
PY3 = sys.version_info[0] == 3

if PY3:
    six_binary_type = bytes
    unicode = (
        str  # Silence `pyflakes` from reporting `undefined name 'unicode'` in Python 3.
    )
    import urllib.request as urllib2
    from http.client import HTTPConnection, HTTPSConnection
    from urllib.error import HTTPError, URLError
    from urllib.parse import urljoin, urlparse
    from urllib.request import Request
else:
    six_binary_type = str
    import urllib2
    from httplib import HTTPConnection, HTTPSConnection
    from urllib2 import HTTPError, Request, URLError
    from urlparse import urljoin, urlparse


log = logging.getLogger(__name__)


# Vendored code from `redo` module
def retrier(attempts=5, sleeptime=10, max_sleeptime=300, sleepscale=1.5, jitter=1):
    """
    This function originates from redo 2.0.3 https://github.com/mozilla-releng/redo
    A generator function that sleeps between retries, handles exponential
    backoff and jitter. The action you are retrying is meant to run after
    retrier yields.
    """
    jitter = jitter or 0  # py35 barfs on the next line if jitter is None
    if jitter > sleeptime:
        # To prevent negative sleep times
        raise Exception(
            "jitter ({}) must be less than sleep time ({})".format(jitter, sleeptime)
        )

    sleeptime_real = sleeptime
    for _ in range(attempts):
        log.debug("attempt %i/%i", _ + 1, attempts)

        yield sleeptime_real

        if jitter:
            sleeptime_real = sleeptime + random.uniform(-jitter, jitter)
            # our jitter should scale along with the sleeptime
            jitter = jitter * sleepscale
        else:
            sleeptime_real = sleeptime

        sleeptime *= sleepscale

        if sleeptime_real > max_sleeptime:
            sleeptime_real = max_sleeptime

        # Don't need to sleep the last time
        if _ < attempts - 1:
            log.debug(
                "sleeping for %.2fs (attempt %i/%i)", sleeptime_real, _ + 1, attempts
            )
            time.sleep(sleeptime_real)


def retry(
    action,
    attempts=5,
    sleeptime=60,
    max_sleeptime=5 * 60,
    sleepscale=1.5,
    jitter=1,
    retry_exceptions=(Exception,),
    cleanup=None,
    args=(),
    kwargs={},
    log_args=True,
):
    """
    This function originates from redo 2.0.3 https://github.com/mozilla-releng/redo
    Calls an action function until it succeeds, or we give up.
    """
    assert callable(action)
    assert not cleanup or callable(cleanup)

    action_name = getattr(action, "__name__", action)
    if log_args and (args or kwargs):
        log_attempt_args = (
            "retry: calling %s with args: %s," " kwargs: %s, attempt #%d",
            action_name,
            args,
            kwargs,
        )
    else:
        log_attempt_args = ("retry: calling %s, attempt #%d", action_name)

    if max_sleeptime < sleeptime:
        log.debug("max_sleeptime %d less than sleeptime %d", max_sleeptime, sleeptime)

    n = 1
    for _ in retrier(
        attempts=attempts,
        sleeptime=sleeptime,
        max_sleeptime=max_sleeptime,
        sleepscale=sleepscale,
        jitter=jitter,
    ):
        try:
            logfn = log.info if n != 1 else log.debug
            logfn_args = log_attempt_args + (n,)
            logfn(*logfn_args)
            return action(*args, **kwargs)
        except retry_exceptions:
            log.debug("retry: Caught exception: ", exc_info=True)
            if cleanup:
                cleanup()
            if n == attempts:
                log.info("retry: Giving up on %s", action_name)
                raise
            continue
        finally:
            n += 1


def retriable(*retry_args, **retry_kwargs):
    """
    This function originates from redo 2.0.3 https://github.com/mozilla-releng/redo
    A decorator factory for retry(). Wrap your function in @retriable(...) to
    give it retry powers!
    """

    def _retriable_factory(func):
        @wraps(func)
        def _retriable_wrapper(*args, **kwargs):
            return retry(func, args=args, kwargs=kwargs, *retry_args, **retry_kwargs)

        return _retriable_wrapper

    return _retriable_factory


# end of vendored code from redo module


def request_has_data(req):
    if PY3:
        return req.data is not None
    return req.has_data()


def get_hexdigest(val):
    return hashlib.sha512(val).hexdigest()


class FileRecordJSONEncoderException(Exception):
    pass


class InvalidManifest(Exception):
    pass


class ExceptionWithFilename(Exception):
    def __init__(self, filename):
        Exception.__init__(self)
        self.filename = filename


class BadFilenameException(ExceptionWithFilename):
    pass


class DigestMismatchException(ExceptionWithFilename):
    pass


class MissingFileException(ExceptionWithFilename):
    pass


class InvalidCredentials(Exception):
    pass


class BadHeaderValue(Exception):
    pass


def parse_url(url):
    url_parts = urlparse(url)
    url_dict = {
        "scheme": url_parts.scheme,
        "hostname": url_parts.hostname,
        "port": url_parts.port,
        "path": url_parts.path,
        "resource": url_parts.path,
        "query": url_parts.query,
    }
    if len(url_dict["query"]) > 0:
        url_dict["resource"] = "%s?%s" % (
            url_dict["resource"],  # pragma: no cover
            url_dict["query"],
        )

    if url_parts.port is None:
        if url_parts.scheme == "http":
            url_dict["port"] = 80
        elif url_parts.scheme == "https":  # pragma: no cover
            url_dict["port"] = 443
    return url_dict


def utc_now(offset_in_seconds=0.0):
    return int(math.floor(calendar.timegm(time.gmtime()) + float(offset_in_seconds)))


def random_string(length):
    return base64.urlsafe_b64encode(os.urandom(length))[:length]


def prepare_header_val(val):
    if isinstance(val, six_binary_type):
        val = val.decode("utf-8")

    if not REQUEST_HEADER_ATTRIBUTE_CHARS.match(val):
        raise BadHeaderValue(  # pragma: no cover
            "header value value={val} contained an illegal character".format(
                val=repr(val)
            )
        )

    return val


def parse_content_type(content_type):  # pragma: no cover
    if content_type:
        return content_type.split(";")[0].strip().lower()
    else:
        return ""


def calculate_payload_hash(algorithm, payload, content_type):  # pragma: no cover
    parts = [
        part if isinstance(part, six_binary_type) else part.encode("utf8")
        for part in [
            "hawk." + str(HAWK_VER) + ".payload\n",
            parse_content_type(content_type) + "\n",
            payload or "",
            "\n",
        ]
    ]

    p_hash = hashlib.new(algorithm)
    for p in parts:
        p_hash.update(p)

    log.debug(
        "calculating payload hash from:\n{parts}".format(parts=pprint.pformat(parts))
    )

    return base64.b64encode(p_hash.digest())


def validate_taskcluster_credentials(credentials):
    if not hasattr(credentials, "__getitem__"):
        raise InvalidCredentials(
            "credentials must be a dict-like object"
        )  # pragma: no cover
    try:
        credentials["clientId"]
        credentials["accessToken"]
    except KeyError:  # pragma: no cover
        etype, val, tb = sys.exc_info()
        raise InvalidCredentials("{etype}: {val}".format(etype=etype, val=val))


def normalize_header_attr(val):
    if isinstance(val, six_binary_type):
        return val.decode("utf-8")
    return val  # pragma: no cover


def normalize_string(
    mac_type,
    timestamp,
    nonce,
    method,
    name,
    host,
    port,
    content_hash,
):
    return "\n".join(
        [
            normalize_header_attr(header)
            # The blank lines are important. They follow what the Node Hawk lib does.
            for header in [
                "hawk." + str(HAWK_VER) + "." + mac_type,
                timestamp,
                nonce,
                method or "",
                name or "",
                host,
                port,
                content_hash or "",
                "",  # for ext which is empty in this case
                "",  # Add trailing new line.
            ]
        ]
    )


def calculate_mac(
    mac_type,
    access_token,
    algorithm,
    timestamp,
    nonce,
    method,
    name,
    host,
    port,
    content_hash,
):
    normalized = normalize_string(
        mac_type, timestamp, nonce, method, name, host, port, content_hash
    )
    log.debug(u"normalized resource for mac calc: {norm}".format(norm=normalized))
    digestmod = getattr(hashlib, algorithm)

    if not isinstance(normalized, six_binary_type):
        normalized = normalized.encode("utf8")

    if not isinstance(access_token, six_binary_type):
        access_token = access_token.encode("ascii")

    result = hmac.new(access_token, normalized, digestmod)
    return base64.b64encode(result.digest())


def make_taskcluster_header(credentials, req):
    validate_taskcluster_credentials(credentials)

    url = req.get_full_url()
    method = req.get_method()
    algorithm = "sha256"
    timestamp = str(utc_now())
    nonce = random_string(6)
    url_parts = parse_url(url)

    content_hash = None
    if request_has_data(req):
        if PY3:
            data = req.data
        else:
            data = req.get_data()
        content_hash = calculate_payload_hash(  # pragma: no cover
            algorithm,
            data,
            # maybe we should detect this from req.headers but we anyway expect json
            content_type="application/json",
        )

    mac = calculate_mac(
        "header",
        credentials["accessToken"],
        algorithm,
        timestamp,
        nonce,
        method,
        url_parts["resource"],
        url_parts["hostname"],
        str(url_parts["port"]),
        content_hash,
    )

    header = u'Hawk mac="{}"'.format(prepare_header_val(mac))

    if content_hash:  # pragma: no cover
        header = u'{}, hash="{}"'.format(header, prepare_header_val(content_hash))

    header = u'{header}, id="{id}", ts="{ts}", nonce="{nonce}"'.format(
        header=header,
        id=prepare_header_val(credentials["clientId"]),
        ts=prepare_header_val(timestamp),
        nonce=prepare_header_val(nonce),
    )

    log.debug("Hawk header for URL={} method={}: {}".format(url, method, header))

    return header


class FileRecord(object):
    def __init__(
        self,
        filename,
        size,
        digest,
        algorithm,
        unpack=False,
        version=None,
        visibility=None,
    ):
        object.__init__(self)
        if "/" in filename or "\\" in filename:
            log.error(
                "The filename provided contains path information and is, therefore, invalid."
            )
            raise BadFilenameException(filename=filename)
        self.filename = filename
        self.size = size
        self.digest = digest
        self.algorithm = algorithm
        self.unpack = unpack
        self.version = version
        self.visibility = visibility

    def __eq__(self, other):
        if self is other:
            return True
        if (
            self.filename == other.filename
            and self.size == other.size
            and self.digest == other.digest
            and self.algorithm == other.algorithm
            and self.version == other.version
            and self.visibility == other.visibility
        ):
            return True
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __str__(self):
        return repr(self)

    def __repr__(self):
        return (
            "%s.%s(filename='%s', size=%s, digest='%s', algorithm='%s', visibility=%r)"
            % (
                __name__,
                self.__class__.__name__,
                self.filename,
                self.size,
                self.digest,
                self.algorithm,
                self.visibility,
            )
        )

    def present(self):
        # Doesn't check validity
        return os.path.exists(self.filename)

    def validate_size(self):
        if self.present():
            return self.size == os.path.getsize(self.filename)
        else:
            log.debug("trying to validate size on a missing file, %s", self.filename)
            raise MissingFileException(filename=self.filename)

    def validate_digest(self):
        if self.present():
            with open(self.filename, "rb") as f:
                return self.digest == digest_file(f, self.algorithm)
        else:
            log.debug("trying to validate digest on a missing file, %s', self.filename")
            raise MissingFileException(filename=self.filename)

    def validate(self):
        if self.size is None or self.validate_size():
            if self.validate_digest():
                return True
        return False

    def describe(self):
        if self.present() and self.validate():
            return "'%s' is present and valid" % self.filename
        elif self.present():
            return "'%s' is present and invalid" % self.filename
        else:
            return "'%s' is absent" % self.filename


def create_file_record(filename, algorithm):
    fo = open(filename, "rb")
    stored_filename = os.path.split(filename)[1]
    fr = FileRecord(
        stored_filename,
        os.path.getsize(filename),
        digest_file(fo, algorithm),
        algorithm,
    )
    fo.close()
    return fr


class FileRecordJSONEncoder(json.JSONEncoder):
    def encode_file_record(self, obj):
        if not issubclass(type(obj), FileRecord):
            err = (
                "FileRecordJSONEncoder is only for FileRecord and lists of FileRecords, "
                "not %s" % obj.__class__.__name__
            )
            log.warn(err)
            raise FileRecordJSONEncoderException(err)
        else:
            rv = {
                "filename": obj.filename,
                "size": obj.size,
                "algorithm": obj.algorithm,
                "digest": obj.digest,
            }
            if obj.unpack:
                rv["unpack"] = True
            if obj.version:
                rv["version"] = obj.version
            if obj.visibility is not None:
                rv["visibility"] = obj.visibility
            return rv

    def default(self, f):
        if issubclass(type(f), list):
            record_list = []
            for i in f:
                record_list.append(self.encode_file_record(i))
            return record_list
        else:
            return self.encode_file_record(f)


class FileRecordJSONDecoder(json.JSONDecoder):

    """I help the json module materialize a FileRecord from
    a JSON file.  I understand FileRecords and lists of
    FileRecords.  I ignore things that I don't expect for now"""

    # TODO: make this more explicit in what it's looking for
    # and error out on unexpected things

    def process_file_records(self, obj):
        if isinstance(obj, list):
            record_list = []
            for i in obj:
                record = self.process_file_records(i)
                if issubclass(type(record), FileRecord):
                    record_list.append(record)
            return record_list
        required_fields = [
            "filename",
            "size",
            "algorithm",
            "digest",
        ]
        if isinstance(obj, dict):
            missing = False
            for req in required_fields:
                if req not in obj:
                    missing = True
                    break

            if not missing:
                unpack = obj.get("unpack", False)
                version = obj.get("version", None)
                visibility = obj.get("visibility", None)
                rv = FileRecord(
                    obj["filename"],
                    obj["size"],
                    obj["digest"],
                    obj["algorithm"],
                    unpack,
                    version,
                    visibility,
                )
                log.debug("materialized %s" % rv)
                return rv
        return obj

    def decode(self, s):
        decoded = json.JSONDecoder.decode(self, s)
        rv = self.process_file_records(decoded)
        return rv


class Manifest(object):

    valid_formats = ("json",)

    def __init__(self, file_records=None):
        self.file_records = file_records or []

    def __eq__(self, other):
        if self is other:
            return True
        if len(self.file_records) != len(other.file_records):
            log.debug("Manifests differ in number of files")
            return False
        # sort the file records by filename before comparing
        mine = sorted((fr.filename, fr) for fr in self.file_records)
        theirs = sorted((fr.filename, fr) for fr in other.file_records)
        return mine == theirs

    def __ne__(self, other):
        return not self.__eq__(other)

    def __deepcopy__(self, memo):
        # This is required for a deep copy
        return Manifest(self.file_records[:])

    def __copy__(self):
        return Manifest(self.file_records)

    def copy(self):
        return Manifest(self.file_records[:])

    def present(self):
        return all(i.present() for i in self.file_records)

    def validate_sizes(self):
        return all(i.validate_size() for i in self.file_records)

    def validate_digests(self):
        return all(i.validate_digest() for i in self.file_records)

    def validate(self):
        return all(i.validate() for i in self.file_records)

    def load(self, data_file, fmt="json"):
        assert fmt in self.valid_formats
        if fmt == "json":
            try:
                self.file_records.extend(
                    json.load(data_file, cls=FileRecordJSONDecoder)
                )
            except ValueError:
                raise InvalidManifest("trying to read invalid manifest file")

    def loads(self, data_string, fmt="json"):
        assert fmt in self.valid_formats
        if fmt == "json":
            try:
                self.file_records.extend(
                    json.loads(data_string, cls=FileRecordJSONDecoder)
                )
            except ValueError:
                raise InvalidManifest("trying to read invalid manifest file")

    def dump(self, output_file, fmt="json"):
        assert fmt in self.valid_formats
        if fmt == "json":
            return json.dump(
                self.file_records,
                output_file,
                indent=2,
                separators=(",", ": "),
                cls=FileRecordJSONEncoder,
            )

    def dumps(self, fmt="json"):
        assert fmt in self.valid_formats
        if fmt == "json":
            return json.dumps(
                self.file_records,
                indent=2,
                separators=(",", ": "),
                cls=FileRecordJSONEncoder,
            )


def digest_file(f, a):
    """I take a file like object 'f' and return a hex-string containing
    of the result of the algorithm 'a' applied to 'f'."""
    h = hashlib.new(a)
    chunk_size = 1024 * 10
    data = f.read(chunk_size)
    while data:
        h.update(data)
        data = f.read(chunk_size)
    name = repr(f.name) if hasattr(f, "name") else "a file"
    log.debug("hashed %s with %s to be %s", name, a, h.hexdigest())
    return h.hexdigest()


def execute(cmd):
    """Execute CMD, logging its stdout at the info level"""
    process = Popen(cmd, shell=True, stdout=PIPE)
    while True:
        line = process.stdout.readline()
        if not line:
            break
        log.info(line.replace("\n", " "))
    return process.wait() == 0


def open_manifest(manifest_file):
    """I know how to take a filename and load it into a Manifest object"""
    if os.path.exists(manifest_file):
        manifest = Manifest()
        with open(manifest_file, "r" if PY3 else "rb") as f:
            manifest.load(f)
            log.debug("loaded manifest from file '%s'" % manifest_file)
        return manifest
    else:
        log.debug("tried to load absent file '%s' as manifest" % manifest_file)
        raise InvalidManifest("manifest file '%s' does not exist" % manifest_file)


def list_manifest(manifest_file):
    """I know how print all the files in a location"""
    try:
        manifest = open_manifest(manifest_file)
    except InvalidManifest as e:
        log.error(
            "failed to load manifest file at '%s': %s"
            % (
                manifest_file,
                str(e),
            )
        )
        return False
    for f in manifest.file_records:
        print(
            "{}\t{}\t{}".format(
                "P" if f.present() else "-",
                "V" if f.present() and f.validate() else "-",
                f.filename,
            )
        )
    return True


def validate_manifest(manifest_file):
    """I validate that all files in a manifest are present and valid but
    don't fetch or delete them if they aren't"""
    try:
        manifest = open_manifest(manifest_file)
    except InvalidManifest as e:
        log.error(
            "failed to load manifest file at '%s': %s"
            % (
                manifest_file,
                str(e),
            )
        )
        return False
    invalid_files = []
    absent_files = []
    for f in manifest.file_records:
        if not f.present():
            absent_files.append(f)
        else:
            if not f.validate():
                invalid_files.append(f)
    if len(invalid_files + absent_files) == 0:
        return True
    else:
        return False


def add_files(manifest_file, algorithm, filenames, version, visibility, unpack):
    # returns True if all files successfully added, False if not
    # and doesn't catch library Exceptions.  If any files are already
    # tracked in the manifest, return will be False because they weren't
    # added
    all_files_added = True
    # Create a old_manifest object to add to
    if os.path.exists(manifest_file):
        old_manifest = open_manifest(manifest_file)
    else:
        old_manifest = Manifest()
        log.debug("creating a new manifest file")
    new_manifest = Manifest()  # use a different manifest for the output
    for filename in filenames:
        log.debug("adding %s" % filename)
        path, name = os.path.split(filename)
        new_fr = create_file_record(filename, algorithm)
        new_fr.version = version
        new_fr.visibility = visibility
        new_fr.unpack = unpack
        log.debug("appending a new file record to manifest file")
        add = True
        for fr in old_manifest.file_records:
            log.debug(
                "manifest file has '%s'"
                % "', ".join([x.filename for x in old_manifest.file_records])
            )
            if new_fr == fr:
                log.info("file already in old_manifest")
                add = False
            elif filename == fr.filename:
                log.error(
                    "manifest already contains a different file named %s" % filename
                )
                add = False
        if add:
            new_manifest.file_records.append(new_fr)
            log.debug("added '%s' to manifest" % filename)
        else:
            all_files_added = False
    # copy any files in the old manifest that aren't in the new one
    new_filenames = set(fr.filename for fr in new_manifest.file_records)
    for old_fr in old_manifest.file_records:
        if old_fr.filename not in new_filenames:
            new_manifest.file_records.append(old_fr)
    if PY3:
        with open(manifest_file, mode="w") as output:
            new_manifest.dump(output, fmt="json")
    else:
        with open(manifest_file, mode="wb") as output:
            new_manifest.dump(output, fmt="json")
    return all_files_added


def touch(f):
    """Used to modify mtime in cached files;
    mtime is used by the purge command"""
    try:
        os.utime(f, None)
    except OSError:
        log.warn("impossible to update utime of file %s" % f)


@contextmanager
@retriable(sleeptime=2)
def request(url, auth_file=None):
    req = Request(url)
    _authorize(req, auth_file)
    with closing(urllib2.urlopen(req)) as f:
        log.debug("opened %s for reading" % url)
        yield f


def fetch_file(base_urls, file_record, grabchunk=1024 * 4, auth_file=None, region=None):
    # A file which is requested to be fetched that exists locally will be
    # overwritten by this function
    fd, temp_path = tempfile.mkstemp(dir=os.getcwd())
    os.close(fd)
    fetched_path = None
    for base_url in base_urls:
        # Generate the URL for the file on the server side
        url = urljoin(base_url, "%s/%s" % (file_record.algorithm, file_record.digest))
        if region is not None:
            url += "?region=" + region

        log.info("Attempting to fetch from '%s'..." % base_url)

        # Well, the file doesn't exist locally.  Let's fetch it.
        try:
            with request(url, auth_file) as f, open(temp_path, mode="wb") as out:
                k = True
                size = 0
                while k:
                    # TODO: print statistics as file transfers happen both for info and to stop
                    # buildbot timeouts
                    indata = f.read(grabchunk)
                    out.write(indata)
                    size += len(indata)
                    if len(indata) == 0:
                        k = False
                log.info(
                    "File %s fetched from %s as %s"
                    % (file_record.filename, base_url, temp_path)
                )
                fetched_path = temp_path
                break
        except (URLError, HTTPError, ValueError):
            log.info(
                "...failed to fetch '%s' from %s" % (file_record.filename, base_url),
                exc_info=True,
            )
        except IOError:  # pragma: no cover
            log.info(
                "failed to write to temporary file for '%s'" % file_record.filename,
                exc_info=True,
            )

    # cleanup temp file in case of issues
    if fetched_path:
        return os.path.split(fetched_path)[1]
    else:
        try:
            os.remove(temp_path)
        except OSError:  # pragma: no cover
            pass
        return None


def clean_path(dirname):
    """Remove a subtree if is exists. Helper for unpack_file()."""
    if os.path.exists(dirname):
        log.info("rm tree: %s" % dirname)
        shutil.rmtree(dirname)


CHECKSUM_SUFFIX = ".checksum"


def validate_tar_member(member, path):
    def _is_within_directory(directory, target):
        real_directory = os.path.realpath(directory)
        real_target = os.path.realpath(target)
        prefix = os.path.commonprefix([real_directory, real_target])
        return prefix == real_directory

    member_path = os.path.join(path, member.name)
    if not _is_within_directory(path, member_path):
        raise Exception("Attempted path traversal in tar file: " + member.name)
    if member.issym():
        link_path = os.path.join(os.path.dirname(member_path), member.linkname)
        if not _is_within_directory(path, link_path):
            raise Exception("Attempted link path traversal in tar file: " + member.name)
    if member.mode & (stat.S_ISUID | stat.S_ISGID):
        raise Exception("Attempted setuid or setgid in tar file: " + member.name)


def safe_extract(tar, path=".", *, numeric_owner=False):
    def _files(tar, path):
        for member in tar:
            validate_tar_member(member, path)
            yield member

    tar.extractall(path, members=_files(tar, path), numeric_owner=numeric_owner)


def unpack_file(filename):
    """Untar `filename`, assuming it is uncompressed or compressed with bzip2,
    xz, gzip, zst, or unzip a zip file. The file is assumed to contain a single
    directory with a name matching the base of the given filename.
    Xz support is handled by shelling out to 'tar'."""
    if os.path.isfile(filename) and tarfile.is_tarfile(filename):
        tar_file, zip_ext = os.path.splitext(filename)
        base_file, tar_ext = os.path.splitext(tar_file)
        clean_path(base_file)
        log.info('untarring "%s"' % filename)
        with tarfile.open(filename) as tar:
            safe_extract(tar)
    elif os.path.isfile(filename) and filename.endswith(".tar.xz"):
        base_file = filename.replace(".tar.xz", "")
        clean_path(base_file)
        log.info('untarring "%s"' % filename)
        # Not using tar -Jxf because it fails on Windows for some reason.
        process = Popen(["xz", "-d", "-c", filename], stdout=PIPE)
        stdout, stderr = process.communicate()
        if process.returncode != 0:
            return False
        fileobj = BytesIO()
        fileobj.write(stdout)
        fileobj.seek(0)
        with tarfile.open(fileobj=fileobj, mode="r|") as tar:
            safe_extract(tar)
    elif os.path.isfile(filename) and filename.endswith(".tar.zst"):
        import zstandard

        base_file = filename.replace(".tar.zst", "")
        clean_path(base_file)
        log.info('untarring "%s"' % filename)
        dctx = zstandard.ZstdDecompressor()
        with dctx.stream_reader(open(filename, "rb")) as fileobj:
            with tarfile.open(fileobj=fileobj, mode="r|") as tar:
                safe_extract(tar)
    elif os.path.isfile(filename) and zipfile.is_zipfile(filename):
        base_file = filename.replace(".zip", "")
        clean_path(base_file)
        log.info('unzipping "%s"' % filename)
        z = zipfile.ZipFile(filename)
        z.extractall()
        z.close()
    else:
        log.error("Unknown archive extension for filename '%s'" % filename)
        return False
    return True


def fetch_files(
    manifest_file,
    base_urls,
    filenames=[],
    cache_folder=None,
    auth_file=None,
    region=None,
):
    # Lets load the manifest file
    try:
        manifest = open_manifest(manifest_file)
    except InvalidManifest as e:
        log.error(
            "failed to load manifest file at '%s': %s"
            % (
                manifest_file,
                str(e),
            )
        )
        return False

    # we want to track files already in current working directory AND valid
    # we will not need to fetch these
    present_files = []

    # We want to track files that fail to be fetched as well as
    # files that are fetched
    failed_files = []
    fetched_files = []

    # Files that we want to unpack.
    unpack_files = []

    # Lets go through the manifest and fetch the files that we want
    for f in manifest.file_records:
        # case 1: files are already present
        if f.present():
            if f.validate():
                present_files.append(f.filename)
                if f.unpack:
                    unpack_files.append(f.filename)
            else:
                # we have an invalid file here, better to cleanup!
                # this invalid file needs to be replaced with a good one
                # from the local cash or fetched from a tooltool server
                log.info(
                    "File %s is present locally but it is invalid, so I will remove it "
                    "and try to fetch it" % f.filename
                )
                os.remove(os.path.join(os.getcwd(), f.filename))

        # check if file is already in cache
        if cache_folder and f.filename not in present_files:
            try:
                shutil.copy(
                    os.path.join(cache_folder, f.digest),
                    os.path.join(os.getcwd(), f.filename),
                )
                log.info(
                    "File %s retrieved from local cache %s" % (f.filename, cache_folder)
                )
                touch(os.path.join(cache_folder, f.digest))

                filerecord_for_validation = FileRecord(
                    f.filename, f.size, f.digest, f.algorithm
                )
                if filerecord_for_validation.validate():
                    present_files.append(f.filename)
                    if f.unpack:
                        unpack_files.append(f.filename)
                else:
                    # the file copied from the cache is invalid, better to
                    # clean up the cache version itself as well
                    log.warn(
                        "File %s retrieved from cache is invalid! I am deleting it from the "
                        "cache as well" % f.filename
                    )
                    os.remove(os.path.join(os.getcwd(), f.filename))
                    os.remove(os.path.join(cache_folder, f.digest))
            except IOError:
                log.info(
                    "File %s not present in local cache folder %s"
                    % (f.filename, cache_folder)
                )

        # now I will try to fetch all files which are not already present and
        # valid, appending a suffix to avoid race conditions
        temp_file_name = None
        # 'filenames' is the list of filenames to be managed, if this variable
        # is a non empty list it can be used to filter if filename is in
        # present_files, it means that I have it already because it was already
        # either in the working dir or in the cache
        if (
            f.filename in filenames or len(filenames) == 0
        ) and f.filename not in present_files:
            log.debug("fetching %s" % f.filename)
            temp_file_name = fetch_file(
                base_urls, f, auth_file=auth_file, region=region
            )
            if temp_file_name:
                fetched_files.append((f, temp_file_name))
            else:
                failed_files.append(f.filename)
        else:
            log.debug("skipping %s" % f.filename)

    # lets ensure that fetched files match what the manifest specified
    for localfile, temp_file_name in fetched_files:
        # since I downloaded to a temp file, I need to perform all validations on the temp file
        # this is why filerecord_for_validation is created

        filerecord_for_validation = FileRecord(
            temp_file_name, localfile.size, localfile.digest, localfile.algorithm
        )

        if filerecord_for_validation.validate():
            # great!
            # I can rename the temp file
            log.info(
                "File integrity verified, renaming %s to %s"
                % (temp_file_name, localfile.filename)
            )
            os.rename(
                os.path.join(os.getcwd(), temp_file_name),
                os.path.join(os.getcwd(), localfile.filename),
            )

            if localfile.unpack:
                unpack_files.append(localfile.filename)

            # if I am using a cache and a new file has just been retrieved from a
            # remote location, I need to update the cache as well
            if cache_folder:
                log.info("Updating local cache %s..." % cache_folder)
                try:
                    if not os.path.exists(cache_folder):
                        log.info("Creating cache in %s..." % cache_folder)
                        os.makedirs(cache_folder, 0o0700)
                    shutil.copy(
                        os.path.join(os.getcwd(), localfile.filename),
                        os.path.join(cache_folder, localfile.digest),
                    )
                    log.info(
                        "Local cache %s updated with %s"
                        % (cache_folder, localfile.filename)
                    )
                    touch(os.path.join(cache_folder, localfile.digest))
                except (OSError, IOError):
                    log.warning(
                        "Impossible to add file %s to cache folder %s"
                        % (localfile.filename, cache_folder),
                        exc_info=True,
                    )
        else:
            failed_files.append(localfile.filename)
            log.error("'%s'" % filerecord_for_validation.describe())
            os.remove(temp_file_name)

    # Unpack files that need to be unpacked.
    for filename in unpack_files:
        if not unpack_file(filename):
            failed_files.append(filename)

    # If we failed to fetch or validate a file, we need to fail
    if len(failed_files) > 0:
        log.error("The following files failed: '%s'" % "', ".join(failed_files))
        return False
    return True


def freespace(p):
    "Returns the number of bytes free under directory `p`"
    if sys.platform == "win32":  # pragma: no cover
        # os.statvfs doesn't work on Windows
        import win32file

        secsPerClus, bytesPerSec, nFreeClus, totClus = win32file.GetDiskFreeSpace(p)
        return secsPerClus * bytesPerSec * nFreeClus
    else:
        r = os.statvfs(p)
        return r.f_frsize * r.f_bavail


def purge(folder, gigs):
    """If gigs is non 0, it deletes files in `folder` until `gigs` GB are free,
    starting from older files.  If gigs is 0, a full purge will be performed.
    No recursive deletion of files in subfolder is performed."""

    full_purge = bool(gigs == 0)
    gigs *= 1024 * 1024 * 1024

    if not full_purge and freespace(folder) >= gigs:
        log.info("No need to cleanup")
        return

    files = []
    for f in os.listdir(folder):
        p = os.path.join(folder, f)
        # it deletes files in folder without going into subfolders,
        # assuming the cache has a flat structure
        if not os.path.isfile(p):
            continue
        mtime = os.path.getmtime(p)
        files.append((mtime, p))

    # iterate files sorted by mtime
    for _, f in sorted(files):
        log.info("removing %s to free up space" % f)
        try:
            os.remove(f)
        except OSError:
            log.info("Impossible to remove %s" % f, exc_info=True)
        if not full_purge and freespace(folder) >= gigs:
            break


def _log_api_error(e):
    if hasattr(e, "hdrs") and e.hdrs["content-type"] == "application/json":
        json_resp = json.load(e.fp)
        log.error(
            "%s: %s" % (json_resp["error"]["name"], json_resp["error"]["description"])
        )
    else:
        log.exception("Error making RelengAPI request:")


def _authorize(req, auth_file):
    is_taskcluster_auth = False

    if not auth_file:
        try:
            taskcluster_env_keys = {
                "clientId": "TASKCLUSTER_CLIENT_ID",
                "accessToken": "TASKCLUSTER_ACCESS_TOKEN",
            }
            auth_content = {k: os.environ[v] for k, v in taskcluster_env_keys.items()}
            is_taskcluster_auth = True
        except KeyError:
            return
    else:
        with open(auth_file) as f:
            auth_content = f.read().strip()
            try:
                auth_content = json.loads(auth_content)
                is_taskcluster_auth = True
            except Exception:
                pass

    if is_taskcluster_auth:
        taskcluster_header = make_taskcluster_header(auth_content, req)
        log.debug("Using taskcluster credentials in %s" % auth_file)
        req.add_unredirected_header("Authorization", taskcluster_header)
    else:
        log.debug("Using Bearer token in %s" % auth_file)
        req.add_unredirected_header("Authorization", "Bearer %s" % auth_content)


def _send_batch(base_url, auth_file, batch, region):
    url = urljoin(base_url, "upload")
    if region is not None:
        url += "?region=" + region
    data = json.dumps(batch)
    if PY3:
        data = data.encode("utf-8")
    req = Request(url, data, {"Content-Type": "application/json"})
    _authorize(req, auth_file)
    try:
        resp = urllib2.urlopen(req)
    except (URLError, HTTPError) as e:
        _log_api_error(e)
        return None
    return json.load(resp)["result"]


def _s3_upload(filename, file):
    # urllib2 does not support streaming, so we fall back to good old httplib
    url = urlparse(file["put_url"])
    cls = HTTPSConnection if url.scheme == "https" else HTTPConnection
    host, port = url.netloc.split(":") if ":" in url.netloc else (url.netloc, 443)
    port = int(port)
    conn = cls(host, port)
    try:
        req_path = "%s?%s" % (url.path, url.query) if url.query else url.path
        with open(filename, "rb") as f:
            content = f.read()
            content_length = len(content)
            f.seek(0)
            conn.request(
                "PUT",
                req_path,
                f,
                {
                    "Content-Type": "application/octet-stream",
                    "Content-Length": str(content_length),
                },
            )
            resp = conn.getresponse()
            resp_body = resp.read()
            conn.close()
        if resp.status != 200:
            raise RuntimeError(
                "Non-200 return from AWS: %s %s\n%s"
                % (resp.status, resp.reason, resp_body)
            )
    except Exception:
        file["upload_exception"] = sys.exc_info()
        file["upload_ok"] = False
    else:
        file["upload_ok"] = True


def _notify_upload_complete(base_url, auth_file, file):
    req = Request(urljoin(base_url, "upload/complete/%(algorithm)s/%(digest)s" % file))
    _authorize(req, auth_file)
    try:
        urllib2.urlopen(req)
    except HTTPError as e:
        if e.code != 409:
            _log_api_error(e)
            return
        # 409 indicates that the upload URL hasn't expired yet and we
        # should retry after a delay
        to_wait = int(e.headers.get("X-Retry-After", 60))
        log.warning("Waiting %d seconds for upload URLs to expire" % to_wait)
        time.sleep(to_wait)
        _notify_upload_complete(base_url, auth_file, file)
    except Exception:
        log.exception("While notifying server of upload completion:")


def upload(manifest, message, base_urls, auth_file, region):
    try:
        manifest = open_manifest(manifest)
    except InvalidManifest:
        log.exception("failed to load manifest file at '%s'")
        return False

    # verify the manifest, since we'll need the files present to upload
    if not manifest.validate():
        log.error("manifest is invalid")
        return False

    if any(fr.visibility is None for fr in manifest.file_records):
        log.error("All files in a manifest for upload must have a visibility set")

    # convert the manifest to an upload batch
    batch = {
        "message": message,
        "files": {},
    }
    for fr in manifest.file_records:
        batch["files"][fr.filename] = {
            "size": fr.size,
            "digest": fr.digest,
            "algorithm": fr.algorithm,
            "visibility": fr.visibility,
        }

    # make the upload request
    resp = _send_batch(base_urls[0], auth_file, batch, region)
    if not resp:
        return None
    files = resp["files"]

    # Upload the files, each in a thread.  This allows us to start all of the
    # uploads before any of the URLs expire.
    threads = {}
    for filename, file in files.items():
        if "put_url" in file:
            log.info("%s: starting upload" % (filename,))
            thd = threading.Thread(target=_s3_upload, args=(filename, file))
            thd.daemon = 1
            thd.start()
            threads[filename] = thd
        else:
            log.info("%s: already exists on server" % (filename,))

    # re-join all of those threads as they exit
    success = True
    while threads:
        for filename, thread in list(threads.items()):
            if not thread.is_alive():
                # _s3_upload has annotated file with result information
                file = files[filename]
                thread.join()
                if file["upload_ok"]:
                    log.info("%s: uploaded" % filename)
                else:
                    log.error(
                        "%s: failed" % filename, exc_info=file["upload_exception"]
                    )
                    success = False
                del threads[filename]

    # notify the server that the uploads are completed.  If the notification
    # fails, we don't consider that an error (the server will notice
    # eventually)
    for filename, file in files.items():
        if "put_url" in file and file["upload_ok"]:
            log.info("notifying server of upload completion for %s" % (filename,))
            _notify_upload_complete(base_urls[0], auth_file, file)

    return success


def send_operation_on_file(data, base_urls, digest, auth_file):
    url = base_urls[0]
    url = urljoin(url, "file/sha512/" + digest)

    data = json.dumps(data)

    req = Request(url, data, {"Content-Type": "application/json"})
    req.get_method = lambda: "PATCH"

    _authorize(req, auth_file)

    try:
        urllib2.urlopen(req)
    except (URLError, HTTPError) as e:
        _log_api_error(e)
        return False
    return True


def change_visibility(base_urls, digest, visibility, auth_file):
    data = [
        {
            "op": "set_visibility",
            "visibility": visibility,
        }
    ]
    return send_operation_on_file(data, base_urls, digest, auth_file)


def delete_instances(base_urls, digest, auth_file):
    data = [
        {
            "op": "delete_instances",
        }
    ]
    return send_operation_on_file(data, base_urls, digest, auth_file)


def process_command(options, args):
    """I know how to take a list of program arguments and
    start doing the right thing with them"""
    cmd = args[0]
    cmd_args = args[1:]
    log.debug("processing '%s' command with args '%s'" % (cmd, '", "'.join(cmd_args)))
    log.debug("using options: %s" % options)

    if cmd == "list":
        return list_manifest(options["manifest"])
    if cmd == "validate":
        return validate_manifest(options["manifest"])
    elif cmd == "add":
        return add_files(
            options["manifest"],
            options["algorithm"],
            cmd_args,
            options["version"],
            options["visibility"],
            options["unpack"],
        )
    elif cmd == "purge":
        if options["cache_folder"]:
            purge(folder=options["cache_folder"], gigs=options["size"])
        else:
            log.critical("please specify the cache folder to be purged")
            return False
    elif cmd == "fetch":
        return fetch_files(
            options["manifest"],
            options["base_url"],
            cmd_args,
            cache_folder=options["cache_folder"],
            auth_file=options.get("auth_file"),
            region=options.get("region"),
        )
    elif cmd == "upload":
        if not options.get("message"):
            log.critical("upload command requires a message")
            return False
        return upload(
            options.get("manifest"),
            options.get("message"),
            options.get("base_url"),
            options.get("auth_file"),
            options.get("region"),
        )
    elif cmd == "change-visibility":
        if not options.get("digest"):
            log.critical("change-visibility command requires a digest option")
            return False
        if not options.get("visibility"):
            log.critical("change-visibility command requires a visibility option")
            return False
        return change_visibility(
            options.get("base_url"),
            options.get("digest"),
            options.get("visibility"),
            options.get("auth_file"),
        )
    elif cmd == "delete":
        if not options.get("digest"):
            log.critical("change-visibility command requires a digest option")
            return False
        return delete_instances(
            options.get("base_url"),
            options.get("digest"),
            options.get("auth_file"),
        )
    else:
        log.critical('command "%s" is not implemented' % cmd)
        return False


def main(argv, _skip_logging=False):
    # Set up option parsing
    parser = optparse.OptionParser()
    parser.add_option(
        "-q",
        "--quiet",
        default=logging.INFO,
        dest="loglevel",
        action="store_const",
        const=logging.ERROR,
    )
    parser.add_option(
        "-v", "--verbose", dest="loglevel", action="store_const", const=logging.DEBUG
    )
    parser.add_option(
        "-m",
        "--manifest",
        default=DEFAULT_MANIFEST_NAME,
        dest="manifest",
        action="store",
        help="specify the manifest file to be operated on",
    )
    parser.add_option(
        "-d",
        "--algorithm",
        default="sha512",
        dest="algorithm",
        action="store",
        help="hashing algorithm to use (only sha512 is allowed)",
    )
    parser.add_option(
        "--digest",
        default=None,
        dest="digest",
        action="store",
        help="digest hash to change visibility for",
    )
    parser.add_option(
        "--visibility",
        default=None,
        dest="visibility",
        choices=["internal", "public"],
        help='Visibility level of this file; "internal" is for '
        "files that cannot be distributed out of the company "
        'but not for secrets; "public" files are available to '
        "anyone without restriction",
    )
    parser.add_option(
        "--unpack",
        default=False,
        dest="unpack",
        action="store_true",
        help="Request unpacking this file after fetch."
        " This is helpful with tarballs.",
    )
    parser.add_option(
        "--version",
        default=None,
        dest="version",
        action="store",
        help="Version string for this file. This annotates the "
        "manifest entry with a version string to help "
        "identify the contents.",
    )
    parser.add_option(
        "-o",
        "--overwrite",
        default=False,
        dest="overwrite",
        action="store_true",
        help="UNUSED; present for backward compatibility",
    )
    parser.add_option(
        "--url",
        dest="base_url",
        action="append",
        help="RelengAPI URL ending with /tooltool/; default "
        "is appropriate for Mozilla",
    )
    parser.add_option(
        "-c", "--cache-folder", dest="cache_folder", help="Local cache folder"
    )
    parser.add_option(
        "-s",
        "--size",
        help="free space required (in GB)",
        dest="size",
        type="float",
        default=0.0,
    )
    parser.add_option(
        "-r",
        "--region",
        help="Preferred AWS region for upload or fetch; " "example: --region=us-west-2",
    )
    parser.add_option(
        "--message",
        help='The "commit message" for an upload; format with a bug number '
        "and brief comment",
        dest="message",
    )
    parser.add_option(
        "--authentication-file",
        help="Use the RelengAPI token found in the given file to "
        "authenticate to the RelengAPI server.",
        dest="auth_file",
    )

    (options_obj, args) = parser.parse_args(argv[1:])

    if not options_obj.base_url:
        tooltool_host = os.environ.get("TOOLTOOL_HOST", "tooltool.mozilla-releng.net")
        taskcluster_proxy_url = os.environ.get("TASKCLUSTER_PROXY_URL")
        if taskcluster_proxy_url:
            tooltool_url = "{}/{}".format(taskcluster_proxy_url, tooltool_host)
        else:
            tooltool_url = "https://{}".format(tooltool_host)

        options_obj.base_url = [tooltool_url]

    # ensure all URLs have a trailing slash
    def add_slash(url):
        return url if url.endswith("/") else (url + "/")

    options_obj.base_url = [add_slash(u) for u in options_obj.base_url]

    # expand ~ in --authentication-file
    if options_obj.auth_file:
        options_obj.auth_file = os.path.expanduser(options_obj.auth_file)

    # Dictionaries are easier to work with
    options = vars(options_obj)

    log.setLevel(options["loglevel"])

    # Set up logging, for now just to the console
    if not _skip_logging:  # pragma: no cover
        ch = logging.StreamHandler()
        cf = logging.Formatter("%(levelname)s - %(message)s")
        ch.setFormatter(cf)
        log.addHandler(ch)

    if options["algorithm"] != "sha512":
        parser.error("only --algorithm sha512 is supported")

    if len(args) < 1:
        parser.error("You must specify a command")

    return 0 if process_command(options, args) else 1


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main(sys.argv))
