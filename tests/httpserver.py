from SimpleHTTPServer import SimpleHTTPRequestHandler
import SocketServer
import os
import sys
from collections import defaultdict

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 0

requests = defaultdict(int)

class CountingRequestHandler(SimpleHTTPRequestHandler):
    def __init__(self, req, client_addr, server):
        SimpleHTTPRequestHandler.__init__(self, req, client_addr, server)

    def do_POST(self):
        global requests
        parts = self.path.split('/')

        if parts[1] == 'reset':
            requests = defaultdict(int)
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.send_header('Content-Length', 0)
            self.end_headers()
            self.wfile.write('')
            return

    def do_GET(self):
        global requests
        parts = self.path.split('?')
        if parts[0] == '/stats':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            if len(parts) > 1:
                body = str(requests['/' + parts[1]])
            else:
                body = ''
                for key, value in requests.iteritems():
                    body += key + ': ' + str(value) + '\n'
            self.send_header('Content-Length', len(body))
            self.end_headers()
            self.wfile.write(body)
            return

        header_list = []
        status = None

        path = self.translate_path(self.path)
        headers = path + '^headers'

        if os.path.isfile(headers):
            try:
                h = open(headers, 'rb')
            except IOError:
                self.send_error(404, "Header file not found")
                return

            header_lines = h.readlines()
            status = int(header_lines[0])
            for header in header_lines[1:]:
                parts = map(lambda x: x.strip(), header.split(':'))
                header_list += [parts]

        if self.headers.get('If-Modified-Since'):
            self.send_response(304)
            self.end_headers()
            return

        if not status or status == 200:
            requests[self.path] += 1

        if status or header_list:
            ctype = self.guess_type(path)
            try:
                # Always read in binary mode. Opening files in text mode may cause
                # newline translations, making the actual size of the content
                # transmitted *less* than the content-length!
                f = open(path, 'rb')
            except IOError:
                self.send_error(404, "File not found")
                return

            try:
                self.send_response(status or 200)
                self.send_header("Content-type", ctype)
                fs = os.fstat(f.fileno())
                self.send_header("Content-Length", str(fs[6]))
                self.send_header("Last-Modified", self.date_time_string(fs.st_mtime))

                for header in header_list:
                    self.send_header(header[0], header[1])

                self.end_headers()

                try:
                    self.copyfile(f, self.wfile)
                finally:
                    f.close()
            except:
                f.close()
                raise
        else:
            SimpleHTTPRequestHandler.do_GET(self)

class MyTCPServer(SocketServer.TCPServer):
    request_queue_size = 2000
    allow_reuse_address = True

httpd = MyTCPServer(("", PORT), CountingRequestHandler)
if not PORT:
    ip, PORT = httpd.server_address

print "serving at port", PORT
sys.stdout.flush()
httpd.serve_forever()
