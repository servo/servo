import urllib


def inline(doc, doctype="html", mime="text/html;charset=utf-8", protocol="http"):
    from .fixtures import server_config, url
    build_url = url(server_config())

    if doctype == "html":
        mime = "text/html;charset=utf-8"
    elif doctype == "xhtml":
        mime = "application/xhtml+xml"
        doc = """<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
  <head>
    <title>XHTML might be the future</title>
  </head>

  <body>
    {}
  </body>
</html>""".format(doc)
    elif doctype == "xml":
        mime = "text/xml"
        doc = """<?xml version="1.0" encoding="UTF-8"?>{}""".format(doc)

    query = {"doc": doc}
    if mime != "text/html;charset=utf8":
        query["content-type"] = mime

    return build_url("/webdriver/tests/support/inline.py",
                     query=urllib.urlencode(query),
                     protocol=protocol)


def iframe(doc):
    return "<iframe src='%s'></iframe>" % inline(doc)


def main(request, response):
    doc = request.GET.first("doc", None)
    content_type = request.GET.first("content-type", "text/html;charset=utf8")
    if doc is None:
        rv = 404, [("Content-Type", "text/plain")], "Missing doc parameter in query"
    else:
        response.headers.update([
            ("Content-Type", content_type),
            ("X-XSS-Protection", "0")
        ])
        rv = doc
    return rv
