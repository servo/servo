import os.path
import time

from wptserve.utils import isomorphic_decode


ETAG = b'"object-fit-stale-image"'
IMAGE_PATH = os.path.join(
    os.path.dirname(isomorphic_decode(__file__)), u"object-fit-stale-image.png")


def main(request, response):
    response.headers.set(b"Cache-Control", b"max-age=0")
    response.headers.set(b"ETag", ETAG)
    response.headers.set(b"Content-Type", b"image/png")

    if request.headers.get(b"If-None-Match") == ETAG:
        time.sleep(2)
        response.status = (304, b"Not Modified")
        return b""

    with open(IMAGE_PATH, "rb") as image:
        return image.read()
