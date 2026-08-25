import time
from base64 import decodebytes

png_response = decodebytes(b'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==')

def main(request, response):
    time.sleep(2)
    return 200, [], png_response
