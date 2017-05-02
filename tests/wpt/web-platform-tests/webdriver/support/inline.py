import urllib

def inline(doc, doctype="html", mime="text/html;charset=utf-8"):
    if doctype == "html":
        mime = "text/html;charset=utf-8"
    elif doctype == "xhtml":
        mime = "application/xhtml+xml"
        doc = r"""<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
  <head>
    <title>XHTML might be the future</title>
  </head>

  <body>
    {}
  </body>
</html>""".format(doc)
    return "data:{},{}".format(mime, urllib.quote(doc))
