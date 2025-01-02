import time

def main(request, response):
    delay_before_header = float(request.GET.first(b"delayBeforeHeader", 0)) / 1000
    delay_before_body = float(request.GET.first(b"delayBeforeBody", 0)) / 1000

    time.sleep(delay_before_header)
    if b"cors" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.write_status_headers()

    time.sleep(delay_before_body)
    response.writer.write_content(b"Body")
