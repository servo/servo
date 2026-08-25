import importlib
header_helpers = importlib.import_module("storage-access-api.resources.header-helpers")

# Sets the `Activate-Storage-Access` response header to a `retry` value
# corresponding to the supplied `allowed_origin`.
def maybe_set_retry(allowed_origin, response):
  if allowed_origin is None:
    return

  if allowed_origin == b'*':
    retry_response = b'retry; allowed-origin=*'
  elif allowed_origin == b'':
    retry_response = b'retry'
  else:
    retry_response = b'retry; allowed-origin=\"' + allowed_origin + b'\"'
  response.headers.set(b'Activate-Storage-Access', retry_response)

def main(request, response):
  request_params = request.GET
  if b'key' in request_params:
    key = request_params.first(b'key')
  # Do not handle requests without a key parameter.
  else:
    return (400, [], b'')

  allowed_origin = request_params.first(b'retry-allowed-origin', None)
  storage_access_status = request.headers.get(b'sec-fetch-storage-access')

  # If a request has been successfully retried and set to active, store its
  # headers under a modified key so they can be retrieved independently of
  # the initial request's headers.
  if storage_access_status == b'active':
      key += b'active'
  maybe_set_retry(allowed_origin, response)

  # Check if the request should redirect.
  header_helpers.maybe_set_redirect(request_params, response, storage_access_status)

  request.server.stash.put(header_helpers.make_stash_key(key, request_params),
                           header_helpers.get_stashable_headers(request.headers),
                           header_helpers.RETRIEVAL_PATH)

  return header_helpers.make_response_body(request_params)