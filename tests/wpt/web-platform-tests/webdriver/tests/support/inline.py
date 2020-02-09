"""Helpers for inlining extracts of documents in tests."""

import urllib


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

def inline(src, doctype="html", mime=None, charset=None, **kwargs):
    """
    Takes a source extract and produces well-formed documents.

    Based on the desired document type, the extract is embedded with
    predefined boilerplate in order to produce well-formed documents.
    The media type and character set may also be individually configured.

    This helper function originally used data URLs, but since these
    are not universally supported (or indeed standardised!) across
    browsers, it now delegates the serving of the document to wptserve.
    This file also acts as a wptserve handler (see the main function
    below) which configures the HTTP response using query parameters.

    This function returns a URL to the wptserve handler, which in turn
    will serve an HTTP response with the requested source extract
    inlined in a well-formed document, and the Content-Type header
    optionally configured using the desired media type and character set.

    Any additional keyword arguments are passed on to the build_url
    function.
    """
    from .fixtures import server_config, url
    build_url = url(server_config())

    if mime is None:
        mime = MIME_TYPES[doctype]
    if charset is None:
        charset = "UTF-8"
    doc = BOILERPLATES[doctype].format(charset=charset, src=src)

    query = {"doc": doc, "mime": mime, "charset": charset}
    return build_url(
        "/webdriver/tests/support/inline.py",
        query=urllib.urlencode(query),
        **kwargs)


def iframe(src, **kwargs):
    """Inlines document extract as the source document of an <iframe>."""
    return "<iframe src='{}'></iframe>".format(inline(src, **kwargs))


def main(request, response):
    doc = request.GET.first("doc", None)
    mime = request.GET.first("mime", None)
    charset = request.GET.first("charset", None)

    if doc is None:
        return 404, [("Content-Type",
                      "text/plain")], "Missing doc parameter in query"

    content_type = []
    if mime is not None:
        content_type.append(mime)
    if charset is not None:
        content_type.append("charset={}".format(charset))

    headers = {"X-XSS-Protection": "0"}
    if len(content_type) > 0:
        headers["Content-Type"] = ";".join(content_type)

    return 200, headers.items(), doc
