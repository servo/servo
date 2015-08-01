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
    url = "http://localhost:{0}/dromaeo/web/?recommended&automated&post_json".format(TEST_SERVER_PORT)
    #url = "http://localhost:{0}/dromaeo/web/?sunspider-string-validate-input&automated&post_json".format(TEST_SERVER_PORT)
    args = [servo_exe, url, "-z", "-f"] 
    return subprocess.Popen(args)

# Print usage if command line args are incorrect
def print_usage():
    print("USAGE: {0} test servo_binary dromaeo_base_dir".format(sys.argv[0]))


# Handle the POST at the end
class RequestHandler(SimpleHTTPServer.SimpleHTTPRequestHandler):
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
        server = BaseHTTPServer.HTTPServer(('', TEST_SERVER_PORT), RequestHandler)

        if cmd == "test":
            print("Testing Dromaeo on Servo!")
            proc = run_servo(servo_exe)
            server.got_post = False
            while not server.got_post:
                server.handle_request()
            print("dromaeo: %s" % server.post_data)
            proc.kill()
        else:
            print_usage()
    else:
        print_usage()
