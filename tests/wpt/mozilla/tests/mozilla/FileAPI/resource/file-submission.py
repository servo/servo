#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.


def fail(msg):
    return ([("Content-Type", "text/plain")], "FAIL: " + msg)


def main(request, response):
    content_type = request.headers.get('Content-Type').split("; ")

    if len(content_type) != 2:
        return fail("content type length is incorrect")

    if content_type[0] != 'multipart/form-data':
        return fail("content type first field is incorrect")

    boundary = content_type[1].strip("boundary=")

    body = "--" + boundary + "\r\nContent-Disposition: form-data; name=\"file-input\"; filename=\"upload.txt\""
    body += "\r\n" + "content-type: text/plain\r\n\r\nHello\r\n--" + boundary + "--"

    if body != request.body:
        return fail("request body doesn't match: " + body + "+++++++" + request.body)

    return ([("Content-Type", "text/plain")], "OK")
