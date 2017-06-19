import os
import time

def main(request, response):
    body = open(os.path.join(os.path.dirname(__file__), "../../css/fonts/ahem/ahem.ttf"), "rb").read()
    delay = float(request.GET.first("ms", 500))
    if delay > 0:
        time.sleep(delay / 1E3);

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("content-length", len(body))
    response.writer.write_header("content-type", "application/octet-stream")
    response.writer.end_headers()

    response.writer.write(body)
