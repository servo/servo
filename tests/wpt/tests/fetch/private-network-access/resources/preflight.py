# This endpoint responds to both preflight requests and the subsequent requests.
#
# Its behavior can be configured with various search/GET parameters, all of
# which are optional:
#
# - treat-as-public-once: Must be a valid UUID if set.
#   If set, then this endpoint expects to receive a non-preflight request first,
#   for which it sets the `Content-Security-Policy: treat-as-public-address`
#   response header. This allows testing "DNS rebinding", where a URL first
#   resolves to the public IP address space, then a non-public IP address space.
# - preflight-uuid: Must be a valid UUID if set, distinct from the value of the
#   `treat-as-public-once` parameter if both are set.
#   If set, then this endpoint expects to receive a preflight request first
#   followed by a regular request, as in the regular CORS protocol. If the
#   `treat-as-public-once` header is also set, it takes precedence: this
#   endpoint expects to receive a non-preflight request first, then a preflight
#   request, then finally a regular request.
#   If unset, then this endpoint expects to receive no preflight request, only
#   a regular (non-OPTIONS) request.
# - preflight-headers: Valid values are:
#   - cors: this endpoint responds with valid CORS headers to preflights. These
#     should be sufficient for non-PNA preflight requests to succeed, but not
#     for PNA-specific preflight requests.
#   - cors+pna: this endpoint responds with valid CORS and PNA headers to
#     preflights. These should be sufficient for both non-PNA preflight
#     requests and PNA-specific preflight requests to succeed.
#   - cors+pna+sw: this endpoint responds with valid CORS and PNA headers and
#     "Access-Control-Allow-Headers: Service-Worker" to preflights. These should
#     be sufficient for both non-PNA preflight requests and PNA-specific
#     preflight requests to succeed. This allows the main request to fetch a
#     service worker script.
#   - unspecified, or any other value: this endpoint responds with no CORS or
#     PNA headers. Preflight requests should fail.
# - final-headers: Valid values are:
#   - cors: this endpoint responds with valid CORS headers to CORS-enabled
#     non-preflight requests. These should be sufficient for non-preflighted
#     CORS-enabled requests to succeed.
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
_ACAPN = ("Access-Control-Allow-Private-Network", "true")
_ACAH = ("Access-Control-Allow-Headers", "Service-Worker")

def _get_response_headers(method, mode, origin):
  acam = ("Access-Control-Allow-Methods", method)

  if mode == b"cors":
    return [acam, _ACAO]

  if mode == b"cors+pna":
    return [acam, _ACAO, _ACAPN]

  if mode == b"cors+pna+sw":
    return [acam, _ACAO, _ACAPN, _ACAH]

  if mode == b"navigation":
    return [
        acam,
        ("Access-Control-Allow-Origin", origin),
        _ACAPN,
        ("Access-Control-Allow-Credentials", "true"),
        ("Access-Control-Allow-Headers", "Upgrade-Insecure-Requests")
    ]

  return []

def _get_expect_single_preflight(request):
  return request.GET.get(b"expect-single-preflight")

def _is_preflight_optional(request):
  return request.GET.get(b"is-preflight-optional") or \
         request.GET.get(b"file-if-no-preflight-received")

def _get_preflight_uuid(request):
  return request.GET.get(b"preflight-uuid")

def _is_loaded_in_fenced_frame(request):
  return request.GET.get(b"is-loaded-in-fenced-frame")

def _should_treat_as_public_once(request):
  uuid = request.GET.get(b"treat-as-public-once")
  if uuid is None:
    # If the search parameter is not given, never treat as public.
    return False

  # If the parameter is given, we treat the request as public only if the UUID
  # has never been seen and stashed.
  result = request.server.stash.take(uuid) is None
  request.server.stash.put(uuid, "")
  return result

def _handle_preflight_request(request, response):
  if _should_treat_as_public_once(request):
    return (400, [], "received preflight for first treat-as-public request")

  uuid = _get_preflight_uuid(request)
  if uuid is None:
    return (400, [], "missing `preflight-uuid` param from preflight URL")

  value = request.server.stash.take(uuid)
  request.server.stash.put(uuid, "preflight")
  if _get_expect_single_preflight(request) and value is not None:
    return (400, [], "received duplicated preflight")

  method = request.headers.get("Access-Control-Request-Method")
  mode = request.GET.get(b"preflight-headers")
  origin = request.headers.get("Origin")
  headers = _get_response_headers(method, mode, origin)

  return (headers, "preflight")

def _final_response_body(request, missing_preflight):
  file_name = None
  if missing_preflight and not request.GET.get(b"is-preflight-optional"):
    file_name = request.GET.get(b"file-if-no-preflight-received")
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
  missing_preflight = False
  if _should_treat_as_public_once(request):
    headers = [("Content-Security-Policy", "treat-as-public-address"),]
  else:
    uuid = _get_preflight_uuid(request)
    if uuid is not None:
      missing_preflight = request.server.stash.take(uuid) is None
      if missing_preflight and not _is_preflight_optional(request):
        return (405, [], "no preflight received")
      request.server.stash.put(uuid, "final")

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

  body = _final_response_body(request, missing_preflight)
  return (headers, body)

def main(request, response):
  try:
    if request.method == "OPTIONS":
      return _handle_preflight_request(request, response)
    else:
      return _handle_final_request(request, response)
  except BaseException as e:
    # Surface exceptions to the client, where they show up as assertion errors.
    return (500, [("X-exception", str(e))], "exception: {}".format(e))
