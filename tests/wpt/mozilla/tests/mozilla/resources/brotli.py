# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


def main(request, response):
    output = '\x1b\x03)\x00\xa4\xcc\xde\xe2\xb3 vA\x00\x0c'
    headers = [("Content-type", "text/plain"),
               ("Content-Encoding", "br"),
               ("Content-Length", len(output))]

    return headers, output
