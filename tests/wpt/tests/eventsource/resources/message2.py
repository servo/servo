import time

def main(request, response):
    response.headers.set(b'Content-Type', b'text/event-stream')
    response.headers.set(b'Cache-Control', b'no-cache')

    response.write_status_headers()

    while True:
        response.writer.write(u"data:msg")
        response.writer.write(u"\n")
        response.writer.write(u"data: msg")
        response.writer.write(u"\n\n")

        response.writer.write(u":")
        response.writer.write(u"\n")

        response.writer.write(u"falsefield:msg")
        response.writer.write(u"\n\n")

        response.writer.write(u"falsefield:msg")
        response.writer.write(u"\n")

        response.writer.write(u"Data:data")
        response.writer.write(u"\n\n")

        response.writer.write(u"data")
        response.writer.write(u"\n\n")

        response.writer.write(u"data:end")
        response.writer.write(u"\n\n")

        time.sleep(2)
