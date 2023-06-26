import time

def main(request, response):
    response.add_required_headers = False # Don't implicitly add HTTP headers
    response.writer.write_status(200)
    response.writer.write_header("Content-Type", "text/html")
    response.writer.end_headers()

    for b in b'<?xml version="1.0" encoding="windows-1251"?':
        response.writer.write(bytes([b]))
        time.sleep(0.05)
    response.writer.write(b'>\n<p>Normal XML declation as slow byte-by-byte trickle</p>\n<p>Test: \xE6</p>\n<p>If &#x0436;, XML decl takes effect</p>')
