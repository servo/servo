import gzip as gzip_module
from cStringIO import StringIO

def main(request, response):
    f = open('resource-timing/resources/resource_timing_test0.xml', 'r')
    output = f.read()

    out = StringIO()
    with gzip_module.GzipFile(fileobj=out, mode="w") as f:
      f.write(output)
    output = out.getvalue()

    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "gzip"),
               ("Content-Length", len(output))]

    return headers, output
