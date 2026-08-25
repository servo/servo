def main(request, response):
    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"text/html")
    return b"""
    <script>
      self.addEventListener('load', evt => {
        self.parent.postMessage({
          origin: '%s',
          referer: '%s',
          'sec-fetch-site': '%s',
          'sec-fetch-mode': '%s',
          'sec-fetch-dest': '%s',
        });
      });
    </script>""" % (request.headers.get(
        b"origin", b"not set"), request.headers.get(b"referer", b"not set"),
                    request.headers.get(b"sec-fetch-site", b"not set"),
                    request.headers.get(b"sec-fetch-mode", b"not set"),
                    request.headers.get(b"sec-fetch-dest", b"not set"))
