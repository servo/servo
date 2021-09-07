"""Helpers for inlining extracts of documents in tests."""

from urllib.parse import urlencode


BOILERPLATES = {
    "html": "<!doctype html>\n<meta charset={charset}>\n{src}",
    "xhtml": """<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
  <head>
    <title>XHTML might be the future</title>
  </head>

  <body>
    {src}
  </body>
</html>""",
    "xml": """<?xml version="1.0" encoding="{charset}"?>\n{src}""",
}
MIME_TYPES = {
    "html": "text/html",
    "xhtml": "application/xhtml+xml",
    "xml": "text/xml",
}


def build_inline(build_url, src, doctype="html", mime=None, charset=None, **kwargs):
    if mime is None:
        mime = MIME_TYPES[doctype]
    if charset is None:
        charset = "UTF-8"
    doc = BOILERPLATES[doctype].format(charset=charset, src=src)

    query = {"doc": doc, "mime": mime, "charset": charset}
    return build_url(
        "/webdriver/tests/support/inline.py",
        query=urlencode(query),
        **kwargs)


def main(request, response):
    doc = request.GET.first(b"doc", None)
    mime = request.GET.first(b"mime", None)
    charset = request.GET.first(b"charset", None)

    if doc is None:
        return 404, [(b"Content-Type",
                      b"text/plain")], b"Missing doc parameter in query"

    content_type = []
    if mime is not None:
        content_type.append(mime)
    if charset is not None:
        content_type.append(b"charset=%s" % charset)

    headers = {b"X-XSS-Protection": b"0"}
    if len(content_type) > 0:
        headers[b"Content-Type"] = b";".join(content_type)

    return 200, headers.items(), doc
