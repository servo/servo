import importlib
keys = importlib.import_module("fedcm.support.keys")

def main(request, response):
  format_type = request.GET.first(b"format", b"default")

  namespace = "/.well-known/web-identity"

  # Clear any existing value first
  request.server.stash.take(keys.WELL_KNOWN_FORMAT_KEY, namespace)

  request.server.stash.put(keys.WELL_KNOWN_FORMAT_KEY, format_type, namespace)

  return namespace