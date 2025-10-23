import re
import os
import json
from wptserve.utils import isomorphic_decode

ENTITY_TAG = b'"sine440-v1"'
# There should be two requests made to this resource:
# 1. A request fetch the whole video. This request should receive a
#    200 Success response with a aged header of value more
#    than 24 hours in the past, simulating an aged proxy cache.
# 2. A subsequent request with a Range header to revalidate the cached content.
#    This request should include an If-Modified-Since / If-Not-Match header
#    and should response with a 304 Not Modified response.

def main(request, response):
    path = os.path.join(request.doc_root, u"media", "sine440.mp3")
    total_size = os.path.getsize(path)

    if_modified_since = request.headers.get(b'If-Modified-Since')
    if_none_match = request.headers.get(b'If-Not-Match')

    if(if_modified_since or if_none_match):
        response.status = 304
        status = 304
        start = 0
        end = 0
    else:
        response.status = 200
        status = 200
        start = 0
        end = total_size

    start = int(start or 0)
    end = int(end or total_size)
    headers = []
    if status == 200:
        headers.append((b"Age", b"86400"))
        headers.append((b"Last-Modified", b"Wed, 21 Oct 2015 07:28:00 GMT"))
        headers.append((b"ETag", ENTITY_TAG))
        video_file = open(path, "rb")
        video_file.seek(start)
        content = video_file.read(end)
    else:
        content = b""
    headers.append((b"Content-Length", str(end - start)))
    headers.append((b"Content-Type", b"audio/mp3"))

    return status, headers, content
