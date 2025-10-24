import re
import os
import json
from wptserve.utils import isomorphic_decode

ENTITY_TAG = b'"sine440-v1"'
# There should be two requests made to this resource:
# 1. A request fetch the whole video. This request should receive a
#    200 Success response with a aged header of value more
#    than 24 hours in the past, simulating an aged proxy cache.
# 2.a A subsequent request with a Range header to revalidate the cached content.
#    This request should include an If-Modified-Since / If-Not-Match header
#    and should response with a 304 Not Modified response.
# 2.b A subsequent request with a Range header to fetch a range of the video.
#    But without Http-Cache revalidation related header.
#    This request should receive a 206 Partial Content response with the
#    requested range of the video.

def main(request, response):
    path = os.path.join(request.doc_root, u"media", "sine440.mp3")
    total_size = os.path.getsize(path)
    if_modified_since = request.headers.get(b'If-Modified-Since')
    if_none_match = request.headers.get(b'If-Not-Match')
    range_header = request.headers.get(b'Range')
    range_header_match = range_header and re.search(r'^bytes=(\d*)-(\d*)$', isomorphic_decode(range_header))

    if range_header_match:
        start, end = range_header_match.groups()
        start = int(start or 0)
        end = int(end or total_size)
        status = 206
    elif if_modified_since or if_none_match:
        status = 304
        start = 0
        end = 0
    else:
        status = 200
        start = 0
        end = total_size

    response.status = status
    headers = []
    if status == 200:
        headers.append((b"Age", b"86400"))
        headers.append((b"Expires", b"Wed, 21 Oct 2015 07:28:00 GMT"))
        headers.append((b"Last-Modified", b"Wed, 21 Oct 2015 07:28:00 GMT"))
        headers.append((b"ETag", ENTITY_TAG))
        video_file = open(path, "rb")
        video_file.seek(start)
        content = video_file.read(end)
    elif status == 206:
        headers.append((b"Content-Range", b"bytes %d-%d/%d" % (start, end, total_size)))
        headers.append((b"Accept-Ranges", b"bytes"))
        headers.append((b"ETag", ENTITY_TAG))
        video_file = open(path, "rb")
        video_file.seek(start)
        end = min(end+1, total_size)
        content = video_file.read(end-start)
    else:
        content = b""
    headers.append((b"Content-Length", str(end - start)))
    headers.append((b"Content-Type", b"audio/mp3"))

    return status, headers, content
