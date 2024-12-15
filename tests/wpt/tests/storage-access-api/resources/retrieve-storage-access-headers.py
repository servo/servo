import hashlib

def main(request, response):
  if b'key' in request.GET:
    key = request.GET.first(b'key')
  # Do not handle requests without a key parameter.
  else:
    return (400, [], b'')

  # Convert the key from String to UUID valid String.
  stash_key = hashlib.md5(key).hexdigest()

  # Handle the header retrieval request.
  headers = request.server.stash.take(stash_key)
  if headers is None:
    return (204, [], b'')
  return (200, [], headers)
