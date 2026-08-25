# This endpoint responds to requests for a target of a (possible) LNA request.
#
# Its behavior can be configured with various search/GET parameters, all of
# which are optional:
#
# - final-headers: Valid values are:
#   - cors: this endpoint responds with valid CORS headers to CORS-enabled
#     non-preflight requests. These should be sufficient for non-preflighted
#     CORS-enabled requests to succeed.
#   - sw: this endpoint responds with a valid Service-Worker header to allow
#     for the request to serve as a Service worker script resource. This is
#     only valid in conjunction with the cors value above.
#   - unspecified: this endpoint responds with no CORS headers to non-preflight
#     requests. This should fail CORS-enabled requests, but be sufficient for
#     no-CORS requests.
#
# The following parameters only affect non-preflight responses:
#
# - redirect: If set, the response code is set to 301 and the `Location`
#   response header is set to this value.
# - mime-type: If set, the `Content-Type` response header is set to this value.
# - file: Specifies a path (relative to this file's directory) to a file. If
#   set, the response body is copied from this file.
# - random-js-prefix: If set to any value, the response body is prefixed with
#   a Javascript comment line containing a random value. This is useful in
#   service worker tests, since service workers are only updated if the new
#   script is not byte-for-byte identical with the old script.
# - body: If set and `file` is not, the response body is set to this value.
#

import os
import random

from wptserve.utils import isomorphic_encode

_ACAO = ("Access-Control-Allow-Origin", "*")
_ACAH = ("Access-Control-Allow-Headers", "Service-Worker")

def _get_response_headers(method, mode, origin):
  acam = ("Access-Control-Allow-Methods", method)

  if mode == b"cors":
    return [acam, _ACAO]

  if mode == b"cors+sw":
    return [acam, _ACAO, _ACAH]

  if mode == b"navigation":
    return [
        acam,
        ("Access-Control-Allow-Origin", origin),
        ("Access-Control-Allow-Credentials", "true"),
    ]

  return []


def _is_loaded_in_fenced_frame(request):
  return request.GET.get(b"is-loaded-in-fenced-frame")

def _final_response_body(request):
  file_name = None
  if file_name is None:
    file_name = request.GET.get(b"file")
  if file_name is None:
    return request.GET.get(b"body") or "success"

  prefix = b""
  if request.GET.get(b"random-js-prefix"):
    value = random.randint(0, 1000000000)
    prefix = isomorphic_encode("// Random value: {}\n\n".format(value))

  path = os.path.join(os.path.dirname(isomorphic_encode(__file__)), file_name)
  with open(path, 'rb') as f:
    contents = f.read()

  return prefix + contents

def _handle_final_request(request, response):
  mode = request.GET.get(b"final-headers")
  origin = request.headers.get("Origin")
  headers = _get_response_headers(request.method, mode, origin)

  redirect = request.GET.get(b"redirect")
  if redirect is not None:
    headers.append(("Location", redirect))
    return (301, headers, b"")

  mime_type = request.GET.get(b"mime-type")
  if mime_type is not None:
    headers.append(("Content-Type", mime_type),)

  if _is_loaded_in_fenced_frame(request):
    headers.append(("Supports-Loading-Mode", "fenced-frame"))

  body = _final_response_body(request)
  return (headers, body)


def main(request, response):
  try:
    return _handle_final_request(request, response)
  except BaseException as e:
    # Surface exceptions to the client, where they show up as assertion errors.
    return (500, [("X-exception", str(e))], "exception: {}".format(e))
