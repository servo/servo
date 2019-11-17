#!/usr/bin/env python

import datetime
import json
import time
from base64 import b64decode

NOTEHDRS = set(['content-type', 'access-control-allow-origin', 'last-modified', 'etag'])
NOBODYSTATUS = set([204, 304])
LOCATIONHDRS = set(['location', 'content-location'])
DATEHDRS = set(['date', 'expires', 'last-modified'])

def main(request, response):
    dispatch = request.GET.first("dispatch", None)
    uuid = request.GET.first("uuid", None)

    if request.method == "OPTIONS":
        return handle_preflight(uuid, request, response)
    if not uuid:
        response.status = (404, "Not Found")
        response.headers.set("Content-Type", "text/plain")
        return "UUID not found"
    if dispatch == 'test':
        return handle_test(uuid, request, response)
    elif dispatch == 'state':
        return handle_state(uuid, request, response)
    response.status = (404, "Not Found")
    response.headers.set("Content-Type", "text/plain")
    return "Fallthrough"

def handle_preflight(uuid, request, response):
    response.status = (200, "OK")
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Access-Control-Allow-Methods", "GET")
    response.headers.set("Access-Control-Allow-Headers", "*")
    response.headers.set("Access-Control-Max-Age", "86400")
    return "Preflight request"

def handle_state(uuid, request, response):
    response.headers.set("Content-Type", "text/plain")
    return json.dumps(request.server.stash.take(uuid))

def handle_test(uuid, request, response):
    server_state = request.server.stash.take(uuid) or []
    try:
        requests = json.loads(b64decode(request.headers.get('Test-Requests', "")))
    except:
        response.status = (400, "Bad Request")
        response.headers.set("Content-Type", "text/plain")
        return "No or bad Test-Requests request header"
    config = requests[len(server_state)]
    if not config:
        response.status = (404, "Not Found")
        response.headers.set("Content-Type", "text/plain")
        return "Config not found"
    noted_headers = {}
    now = time.time()
    for header in config.get('response_headers', []):
        if header[0].lower() in LOCATIONHDRS: # magic locations
            if (len(header[1]) > 0):
                header[1] = "%s&target=%s" % (request.url, header[1])
            else:
                header[1] = request.url
        if header[0].lower() in DATEHDRS and isinstance(header[1], int):  # magic dates
            header[1] = http_date(now, header[1])
        response.headers.set(header[0], header[1])
        if header[0].lower() in NOTEHDRS:
            noted_headers[header[0].lower()] = header[1]
    state = {
        'now': now,
        'request_method': request.method,
        'request_headers': dict([[h.lower(), request.headers[h]] for h in request.headers]),
        'response_headers': noted_headers
    }
    server_state.append(state)
    request.server.stash.put(uuid, server_state)

    if "access-control-allow-origin" not in noted_headers:
        response.headers.set("Access-Control-Allow-Origin", "*")
    if "content-type" not in noted_headers:
        response.headers.set("Content-Type", "text/plain")
    response.headers.set("Server-Request-Count", len(server_state))

    code, phrase = config.get("response_status", [200, "OK"])
    if config.get("expected_type", "").endswith('validated'):
        ref_hdrs = server_state[0]['response_headers']
        previous_lm = ref_hdrs.get('last-modified', False)
        if previous_lm and request.headers.get("If-Modified-Since", False) == previous_lm:
            code, phrase = [304, "Not Modified"]
        previous_etag = ref_hdrs.get('etag', False)
        if previous_etag and request.headers.get("If-None-Match", False) == previous_etag:
            code, phrase = [304, "Not Modified"]
        if code != 304:
            code, phrase = [999, '304 Not Generated']
    response.status = (code, phrase)

    content = config.get("response_body", uuid)
    if code in NOBODYSTATUS:
        return ""
    return content


def get_header(headers, header_name):
    result = None
    for header in headers:
        if header[0].lower() == header_name.lower():
            result = header[1]
    return result

WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
MONTHS = [None, 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul',
          'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

def http_date(now, delta_secs=0):
    date = datetime.datetime.utcfromtimestamp(now + delta_secs)
    return "%s, %.2d %s %.4d %.2d:%.2d:%.2d GMT" % (
        WEEKDAYS[date.weekday()],
        date.day,
        MONTHS[date.month],
        date.year,
        date.hour,
        date.minute,
        date.second)
