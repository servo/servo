import gzip as gzip_module
from cStringIO import StringIO
import os

def main(request, response):
    dir_path = os.path.dirname(os.path.realpath(__file__))
    file_path = os.path.join(dir_path, 'resource_timing_test0.xml')
    f = open(file_path, 'r')
    output = f.read()

    out = StringIO()
    with gzip_module.GzipFile(fileobj=out, mode="w") as f:
      f.write(output)
    output = out.getvalue()

    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "gzip"),
               ("Content-Length", len(output))]

    return headers, output
