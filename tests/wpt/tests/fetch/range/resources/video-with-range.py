import re
import os
import json
from wptserve.utils import isomorphic_decode

def main(request, response):
    path = os.path.join(request.doc_root, u"media", "sine440.mp3")
    total_size = os.path.getsize(path)
    rewrites = json.loads(request.GET.first(b'rewrites', '[]'))
    range_header = request.headers.get(b'Range')
    range_header_match = range_header and re.search(r'^bytes=(\d*)-(\d*)$', isomorphic_decode(range_header))
    start = None
    end = None
    if range_header_match:
        response.status = 206
        start, end = range_header_match.groups()
    if range_header:
        status = 206
    else:
        status = 200
    for rewrite in rewrites:
        req_start, req_end = rewrite['request']
        if start == req_start or req_start == '*':
            if end == req_end or req_end == '*':
                if 'response' in rewrite:
                    start, end = rewrite['response']
                if 'status' in rewrite:
                    status = rewrite['status']

    start = int(start or 0)
    end = int(end or total_size)
    headers = []
    if status == 206:
        headers.append((b"Content-Range", b"bytes %d-%d/%d" % (start, end - 1, total_size)))
        headers.append((b"Accept-Ranges", b"bytes"))

    headers.append((b"Content-Type", b"audio/mp3"))
    headers.append((b"Content-Length", str(end - start)))
    headers.append((b"Cache-Control", b"no-cache"))
    video_file = open(path, "rb")
    video_file.seek(start)
    content = video_file.read(end)
    return status, headers, content
