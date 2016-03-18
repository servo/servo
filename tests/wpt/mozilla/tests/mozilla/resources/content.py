# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


def main(request, response):
    headers = [("Content-type", "text/plain"),
               ("X-Request-Method", request.method),
               ("X-Request-Query", request.url_parts.query if request.url_parts.query else "NO"),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO"))]

    if "content" in request.GET:
        content = request.GET.first("content")
    else:
        content = request.body

    return headers, content
