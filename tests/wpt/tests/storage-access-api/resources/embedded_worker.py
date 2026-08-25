from cookies.resources.helpers import setNoCacheAndCORSHeaders

# This worker messages how many connections have been made and checks what cookies are available.
def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/javascript")
    cookie_header = request.headers.get(b"Cookie", b"")
    document = b"""
"use strict";

self.onmessage = async (message) => {
  function reply(data) {
    self.postMessage({data});
  }

  switch (message.data.command) {
    case "fetch": {
      const response = await fetch(message.data.url, {mode: 'cors', credentials: 'include'})
        .then((resp) => resp.text());
      reply(response);
      break;
    }
    case "load": {
      reply(\"""" + cookie_header + b"""");
      break;
    }
    default:
  }
};
"""
    return headers, document
