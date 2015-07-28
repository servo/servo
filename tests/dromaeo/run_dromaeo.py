#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import re
import subprocess
import sys
import BaseHTTPServer
import SimpleHTTPServer
import SocketServer
import threading
import urlparse

# Port to run the HTTP server on for Dromaeo.
TEST_SERVER_PORT = 8192

# Run servo and print / parse the results for a specific jQuery test module.
def run_servo(servo_exe):
    url = "http://localhost:{0}/?recommended&automated&post_json".format(TEST_SERVER_PORT)
    #args = [servo_exe, url, "-z", "-f"] 
    args = [servo_exe, url, "-z"] 
    return subprocess.Popen(args)

# Print usage if command line args are incorrect
def print_usage():
    print("USAGE: {0} test servo_binary dromaeo_base_dir".format(sys.argv[0]))


# A simple HTTP server to serve up the test suite
class Server(SocketServer.ThreadingMixIn,
                   BaseHTTPServer.HTTPServer):
    allow_reuse_address = True
    got_post = False

class RequestHandler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    # TODO(gw): HACK copy the fixed version from python
    # main repo - due to https://bugs.python.org/issue23112
    def send_head(self):
        path = self.translate_path(self.path)
        f = None
        if os.path.isdir(path):
            parts = urlparse.urlsplit(self.path)
            if not parts.path.endswith('/'):
                # redirect browser - doing basically what apache does
                self.send_response(301)
                new_parts = (parts[0], parts[1], parts[2] + '/',
                             parts[3], parts[4])
                new_url = urlparse.urlunsplit(new_parts)
                self.send_header("Location", new_url)
                self.end_headers()
                return None
            for index in "index.html", "index.htm":
                index = os.path.join(path, index)
                if os.path.exists(index):
                    path = index
                    break
            else:
                return self.list_directory(path)
        ctype = self.guess_type(path)
        try:
            # Always read in binary mode. Opening files in text mode may cause
            # newline translations, making the actual size of the content
            # transmitted *less* than the content-length!
            f = open(path, 'rb')
        except IOError:
            self.send_error(404, "File not found")
            return None
        try:
            self.send_response(200)
            self.send_header("Content-type", ctype)
            fs = os.fstat(f.fileno())
            self.send_header("Content-Length", str(fs[6]))
            self.send_header("Last-Modified", self.date_time_string(fs.st_mtime))
            self.end_headers()
            return f
        except:
            f.close()
            raise
    def do_POST(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write("<HTML>POST OK.<BR><BR>");
        length = int(self.headers.getheader('content-length'))
        parameters = urlparse.parse_qs(self.rfile.read(length))
        self.server.got_post = True
        self.server.post_data = parameters['data']
    def log_message(self, format, *args):
        return

if __name__ == '__main__':
    if len(sys.argv) == 4:
        cmd = sys.argv[1]
        servo_exe = sys.argv[2]
        base_dir = sys.argv[3]
        os.chdir(base_dir)

        # Ensure servo binary can be found
        if not os.path.isfile(servo_exe):
            print("Unable to find {0}. This script expects an existing build of Servo.".format(servo_exe))
            sys.exit(1)

        # Start the test server
        server = Server(('', TEST_SERVER_PORT), RequestHandler)

        if cmd == "test":
            print("Testing Dromaeo on Servo!")
            proc = run_servo(servo_exe)
            while not server.got_post:
                server.handle_request()
            print("dromaeo: %s" % server.post_data)
            proc.kill()
        else:
            print_usage()
    else:
        print_usage()
