import time

def main(request, response):
    # Sleep for 500ms to delay onload.
    time.sleep(0.5)
    response.headers.set("Cache-Control", "no-cache, must-revalidate");
    response.headers.set("Location", "data:image/gif;base64,R0lGODlhAQABAJAAAMjIyAAAACwAAAAAAQABAAACAgQBADs%3D");
