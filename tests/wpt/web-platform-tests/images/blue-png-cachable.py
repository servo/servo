import os
import time

def main(request, response):
  """Serves the contents in blue.png but with a Cache-Control header.

  Emits a Cache-Control header with max-age set to 1h to allow the browser
  cache the image. Used for testing behaviors involving caching logics.
  """
  image_path = os.path.join(os.path.dirname(__file__), "blue.png")
  response.headers.set("Cache-Control", "max-age=3600")
  response.headers.set("Content-Type", "image/png")
  response.content = open(image_path, mode='rb').read()
