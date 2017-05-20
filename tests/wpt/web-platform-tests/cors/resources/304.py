#!/usr/bin/env python

# A header used to correlate requests and responses
state_header = "content-language"

# Static ETag to use (and expect)
etag = "abcdef"

def error(msg):
    return (299, "Client Error"), [
        ('content-type', 'text/plain'),
        ('access-control-allow-origin', "*"),
        ('access-control-expose-headers', state_header),
        ('cache-control', 'no-store')
    ], msg

def main(request, response):
    headers = []

    inm = request.headers.get('if-none-match', None)
    raw_req_num = request.headers.get(state_header, None)
    if raw_req_num == None:
        return error("no req_num header in request")
    else:
        req_num = int(raw_req_num)
        if req_num > 8:
            return error("req_num %s out of range" % req_num)

    headers.append(("Access-Control-Expose-Headers", state_header))
    headers.append((state_header, req_num))
    headers.append(("A", req_num))
    headers.append(("B", req_num))

    if req_num % 2:  # odd requests are the first in a test pair
        if inm:
            # what are you doing here? This should be a fresh request.
            return error("If-None-Match on first request")
        else:
            status = 200, "OK"
            headers.append(("Access-Control-Allow-Origin", "*"))
            headers.append(("Content-Type", "text/plain"))
            headers.append(("Cache-Control", "private, max-age=3, must-revalidate"))
            headers.append(("ETag", etag))
            return status, headers, "Success"
    else:  # even requests are the second in a pair, and should have a good INM.
        if inm != etag:
            # Bad browser.
            if inm == None:
                return error("If-None-Match missing")
            else:
                return error("If-None-Match '%s' mismatches")
        else:
            if req_num == 2:
                pass  # basic, vanilla check
            elif req_num == 4:
                headers.append(("Access-Control-Expose-Headers", "a, b"))
            elif req_num == 6:
                headers.append(("Access-Control-Expose-Headers", "a"))
            elif req_num == 8:
                headers.append(("Access-Control-Allow-Origin", "other.origin.example:80"))
            status = 304, "Not Modified"
            return status, headers, ""
