import time

def main(request, response):
    # Sleep for 1sec
    time.sleep(1)
    response.headers.set(b"Cache-Control", b"no-cache, no-store, must-revalidate");
