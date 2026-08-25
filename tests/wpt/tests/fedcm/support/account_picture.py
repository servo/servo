import importlib
from base64 import decodebytes
keys = importlib.import_module("fedcm.support.keys")
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.pictureCheck(request)
  if request_error:
   return request_error

  counter = request.server.stash.take(keys.ACCOUNT_PICTURE_COUNTER_KEY)
  try:
    counter = int(counter) + 1
  except (TypeError, ValueError):
    counter = 1

  request.server.stash.put(keys.ACCOUNT_PICTURE_COUNTER_KEY, str(counter).encode())

  response.headers.set(b"Content-Type", b"image/png")
  response.headers.set(b"Cache-Control", b"max-age=3600")
  # Return minimum valid PNG
  png_response = decodebytes(b'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==')
  return png_response
