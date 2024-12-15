import hashlib
import json

RETRIEVAL_PATH = '/storage-access-api/resources/retrieve-storage-access-headers.py'

def get_stashable_headers(headers):
  def bytes_to_strings(d):
    # Recursively convert bytes to strings in `d`.
    if isinstance(d, (tuple,list,set)):
      return [bytes_to_strings(x) for x in d]
    elif isinstance(d, bytes):
      return d.decode()
    elif isinstance(d, dict):
      return {bytes_to_strings(k): bytes_to_strings(v) for (k, v) in d.items()}
    return d
  return json.dumps(bytes_to_strings(headers))

# Makes a string representation of the hash generated for the given key.
def make_stash_key(key, request_params):
  # Redirected requests stash their headers at a different key.
  key += b'redirected' if b'redirected' in request_params else b''
  return hashlib.md5(key).hexdigest()

# Sets the response to redirect to the location passed by the request if its
# parameters contain a `redirect-location` or a `once-active-redirect-location`
# with an active `storage_access_status`.
def maybe_set_redirect(request_params, response, storage_access_status):
  if storage_access_status == b'active':
    location = request_params.first(b'once-active-redirect-location', '')
  else:
    location = request_params.first(b'redirect-location', '')

  if location:
    response.status = 302
    response.headers.set(b'Location', location)

# Returns an HTML body with an embedded responder if one is included in
# `request_params`, otherwise returns an empty byte-string.
def make_response_body(request_params):
  script = request_params.first(b'script', None)
  if script is None:
    return b''
  return b"""
    <!DOCTYPE html>
  <meta charset="utf-8">
  <title>Subframe with script</title>
  <script src="/resources/testharness.js"></script>
  <script src="/resources/testdriver.js"></script>
  <script src="/resources/testdriver-vendor.js"></script>
  <script>
    var should_ack_load = false;
  </script>

  <body>
  <script src="%s"></script>
  </body>

  """ % (script)