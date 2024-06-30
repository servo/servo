# 'import fedcm.support.keys' does not work.
import importlib
keys = importlib.import_module("fedcm.support.keys")

def main(request, response):
  client_metadata_url = "/fedcm/support/client_metadata.py"
  counter = request.server.stash.take(keys.CLIENT_METADATA_COUNTER_KEY,
                                      client_metadata_url)

  try:
    counter = counter.decode()
  except (UnicodeDecodeError, AttributeError):
    pass

  return counter
