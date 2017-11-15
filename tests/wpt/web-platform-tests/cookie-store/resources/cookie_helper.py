#!/usr/bin/env python
# -*- coding: utf-8 -*-

# Active wptserve handler for cookie operations.
#
# This must support the following requests:
#
# - GET with the following query parameters:
#   - charset: (optional) character set for response (default: utf-8)
#   A cookie: request header (if present) is echoed in the body with a
#   cookie= prefix followed by the urlencoded bytes from the header.
#   Used to inspect the cookie jar from an HTTP request header context.
# - POST with form-data in the body and the following query-or-form parameters:
#   - set-cookie: (optional; repeated) echoed in the set-cookie: response
#     header and also echoed in the body with a set-cookie= prefix
#     followed by the urlencoded bytes from the parameter; multiple occurrences
#     are CRLF-delimited.
#   Used to set cookies from an HTTP response header context.
#
# The response has 200 status and content-type: text/plain; charset=<charset>
import cgi, encodings, os, re, sys, urllib

# NOTE: These are intentionally very lax to permit testing
DISALLOWED_IN_COOKIE_NAME_RE = re.compile(r'[;\0-\x1f\x7f]');
DISALLOWED_IN_HEADER_RE = re.compile(r'[\0-\x1f\x7f]');

# Ensure common charset names do not end up with different
# capitalization or punctuation
CHARSET_OVERRIDES = {
    encodings.codecs.lookup(charset).name: charset
    for charset in ('utf-8', 'iso-8859-1',)
}

def main(request, response):
  assert request.method in (
      'GET',
      'POST',
  ), 'request method was neither GET nor POST: %r' % request.method
  qd = (request.url.split('#')[0].split('?', 1) + [''])[1]
  if request.method == 'POST':
    qd += '&' + request.body
  args = cgi.parse_qs(qd, keep_blank_values = True)
  charset = encodings.codecs.lookup(args.get('charset', ['utf-8'])[-1]).name
  charset = CHARSET_OVERRIDES.get(charset, charset)
  headers = [('content-type', 'text/plain; charset=' + charset)]
  body = []
  if request.method == 'POST':
    for set_cookie in args.get('set-cookie', []):
      if '=' in set_cookie.split(';', 1)[0]:
        name, rest = set_cookie.split('=', 1)
        assert re.search(
            DISALLOWED_IN_COOKIE_NAME_RE,
            name
        ) is None, 'name had disallowed characters: %r' % name
      else:
        rest = set_cookie
      assert re.search(
          DISALLOWED_IN_HEADER_RE,
          rest
      ) is None, 'rest had disallowed characters: %r' % rest
      headers.append(('set-cookie', set_cookie))
      body.append('set-cookie=' + urllib.quote(set_cookie, ''))
  else:
    cookie = request.headers.get('cookie')
    if cookie is not None:
      body.append('cookie=' + urllib.quote(cookie, ''))
  body = '\r\n'.join(body)
  headers.append(('content-length', str(len(body))))
  return 200, headers, body
