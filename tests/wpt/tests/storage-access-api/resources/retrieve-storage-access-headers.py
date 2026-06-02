import importlib
header_helpers = importlib.import_module("storage-access-api.resources.header-helpers")

def main(request, response):
  if b'key' in request.GET:
    key = request.GET.first(b'key')
  # Do not handle requests without a key parameter.
  else:
    return (400, [], b'')

  # Convert the key from String to UUID valid String.
  stash_key = header_helpers.make_stash_key(key, request.GET)

  # Handle the header retrieval request.
  headers = request.server.stash.take(stash_key)
  if headers is None:
    return (204, [], b'')
  return (200, [], headers)
