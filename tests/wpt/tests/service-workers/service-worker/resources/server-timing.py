from base64 import decodebytes


def main(request, response):
    headers = [
        (b"Content-Type", b"image/png"),
        (b"Server-Timing", b"metric;dur=123.4;desc=\"description\""),
    ]
    if b"tao" in request.GET:
        headers.append((b"Timing-Allow-Origin", b"*"))
    if b"cors" in request.GET:
        headers.append((b"Access-Control-Allow-Origin", b"*"))

    # 1x1 transparent PNG
    png_data = decodebytes(
        b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk"
        b"YAAAAAYAAjCB0C8AAAAASUVORK5CYII=")
    return headers, png_data
