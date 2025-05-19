import importlib
keys = importlib.import_module("fedcm.support.keys")

def main(request, response):
  account_picture_error_url = "/fedcm/support/account_picture_uncached.py"

  counter = request.server.stash.take(keys.ACCOUNT_PICTURE_UNCACHED_COUNTER_KEY, account_picture_error_url)
  try:
    counter = counter.decode()
  except (UnicodeDecodeError, AttributeError):
    counter = 0

  return str(counter)
