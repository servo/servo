import os.path

from wptserve.utils import isomorphic_decode

def main(request, response):

    key = request.GET['id']
    alreadyServedRequest = False
    try:
      alreadyServedRequest = request.server.stash.take(key)
    except (KeyError, ValueError) as e:
      pass

    if alreadyServedRequest:
      body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"../../../../images/red.png"), u"rb").read()
    else:
      request.server.stash.put(key, True);
      body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"../../../../images/green.png"), u"rb").read()
      pass

    response.writer.write_status(200)
    response.writer.write_header(b"etag", "abcdef")
    response.writer.write_header(b"content-length", len(body))
    response.writer.write_header(b"content-type", "image/png")
    response.writer.write_header(b"cache-control", "public, max-age=31536000, no-cache")
    response.writer.end_headers()

    response.writer.write(body)
