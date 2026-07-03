import os
from urllib.parse import urlsplit

from wptserve.ranges import RangeParser
from wptserve.utils import HTTPException


def resolve_path(request, path):
    if os.path.isfile(path):
        return path

    parsed = urlsplit(path)
    if parsed.scheme in ("http", "https"):
        path = parsed.path

    if path.startswith("/"):
        doc_root = os.path.normpath(request.doc_root)
        candidate = os.path.normpath(os.path.join(doc_root, path.lstrip("/")))
        if candidate == doc_root or candidate.startswith(doc_root + os.path.sep):
            return candidate

    return path


CONTENT_TYPES = {
    ".webm": b"video/webm",
    ".mp4": b"video/mp4",
    ".m4v": b"video/mp4",
    ".ogg": b"video/ogg",
    ".ogv": b"video/ogg",
    ".mov": b"video/quicktime",
    ".mkv": b"video/x-matroska",
}


def content_type_for(path):
    ext = os.path.splitext(path)[1].lower()
    return CONTENT_TYPES.get(ext)


def send_range_not_satisfiable(response, size):
    response.status = 416
    response.headers.set(b"Content-Range", f"bytes */{size}".encode("ascii"))
    response.headers.set(b"Content-Length", b"0")
    response.content = b""


def main(request, response):
    """Serve a local video file passed via the `path` query parameter."""

    path = request.GET.first(b"path", None)

    if path is None:
        response.status = 400
        response.content = b"Missing required `path` query parameter"
        return

    path = resolve_path(request, path.decode("utf-8"))

    content_type = content_type_for(path)
    if content_type is None:
        response.status = 400
        response.content = b"Unsupported video file extension"
        return

    with open(path, "rb") as f:
        content = f.read()

    size = len(content)

    response.headers.set(b"Content-Type", content_type)

    range_header = request.headers.get(b"Range")
    if range_header:
        try:
            ranges = RangeParser()(range_header, size)
        except HTTPException:
            send_range_not_satisfiable(response, size)
            return

        # Only single-range requests are supported; <video> never asks for more.
        byte_range = ranges[0]
        response.status = 206
        response.headers.set(
            b"Content-Range", byte_range.header_value().encode("ascii")
        )
        response.headers.set(
            b"Content-Length", str(byte_range.upper - byte_range.lower).encode("ascii")
        )
        response.content = content[byte_range.lower : byte_range.upper]
        return

    response.headers.set(b"Content-Length", str(size).encode("ascii"))
    response.content = content
