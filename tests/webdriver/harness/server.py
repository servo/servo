# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Example usage from a test:
#
#   def handler(self):
#     # returns the HTTP status code, a list of HTTP header names and value pairs,
#     # and the response body
#     return 200, [('Content-Type', 'text/html')], '<html><body>hi there</body></html>'
#
#   import server
#   server.serve(handler)
#   # run a test that involves Servo fetching from http://localhost:8001
#   server.stop()
#
#
# The self argument to handler is an instance of BaseHTTPRequestHandler.
# (https://docs.python.org/2/library/basehttpserver.html#BaseHTTPServer.BaseHTTPRequestHandler)

from BaseHTTPServer import BaseHTTPRequestHandler, HTTPServer
import threading


class TestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        status, headers, body = self.handler()
        self.send_response(status)
        for header in headers:
            self.send_header(header[0], header[1])
        self.end_headers()
        self.wfile.write(body)


class TestServer(HTTPServer):
    def shutdown(self):
        self.socket.close()
        HTTPServer.shutdown(self)

httpd = None


def serve(handler):
    global httpd
    base_handler = TestHandler
    base_handler.handler = handler

    httpd = TestServer(("", 8001), base_handler)
    httpd_thread = threading.Thread(target=httpd.serve_forever)
    httpd_thread.setDaemon(True)
    httpd_thread.start()


def stop():
    httpd.shutdown()
