import time

def main(request, response):
    # Sleep for 500ms to delay onload.
    time.sleep(0.5)
    response.headers.set(b"Cache-Control", b"no-cache, must-revalidate");
    response.headers.set(b"Location", b"data:image/gif;base64,R0lGODlhAQABAJAAAMjIyAAAACwAAAAAAQABAAACAgQBADs%3D");
