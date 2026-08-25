import importlib
header_helpers = importlib.import_module("storage-access-api.resources.header-helpers")

def main(request, response):
  request_params = request.GET
  if b'key' in request_params:
    key = request_params.first(b'key')
  # Do not handle requests without a key parameter.
  else:
    return (400, [], b'')

  # Handle load requests.
  if b'load' in request_params:
    response.headers.set(b'Activate-Storage-Access', b'load')

  request.server.stash.put(header_helpers.make_stash_key(key, request_params),
                           header_helpers.get_stashable_headers(request.headers),
                           header_helpers.RETRIEVAL_PATH)

  return header_helpers.make_response_body(request_params)