import importlib
from urllib.parse import urlsplit

# 'import fedcm.support.keys' does not work.
keys = importlib.import_module("fedcm.support.keys")

def main(request, response):
  root_manifest_url = "/.well-known/web-identity"

  # Clear stash so that a new value can be written.
  request.server.stash.take(keys.MANIFEST_URL_IN_MANIFEST_LIST_KEY, root_manifest_url)

  request.server.stash.put(keys.MANIFEST_URL_IN_MANIFEST_LIST_KEY,
                           request.GET.first(b"manifest_url", b""),
                           root_manifest_url)

  return root_manifest_url
