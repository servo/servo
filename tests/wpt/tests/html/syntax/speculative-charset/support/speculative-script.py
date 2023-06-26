import time

def main(request, response):
    response.add_required_headers = False # Don't implicitly add HTTP headers
    response.writer.write_status(200)
    response.writer.write_header("Content-Type", "text/html")
    response.writer.end_headers()

    response.writer.write(b'<!DOCTYPE html><script src="script.py?uuid=%s&character=&#x03B6;"></script>' % request.GET[b"uuid"]);
    time.sleep(0.2)
    response.writer.write(b'<meta charset="windows-1251"><p>Test: \xE6</p>');
