import time

def main(request, response):
    response.headers.set('Content-Type', 'text/event-stream')
    response.headers.set('Cache-Control', 'no-cache')

    response.explicit_flush = True
    response.write_status_headers()

    while True:
        response.writer.write("data:msg")
        response.writer.write("\n")
        response.writer.write("data: msg")
        response.writer.write("\n\n")

        response.writer.write(":")
        response.writer.write("\n")

        response.writer.write("falsefield:msg")
        response.writer.write("\n\n")

        response.writer.write("falsefield:msg")
        response.writer.write("\n")

        response.writer.write("Data:data")
        response.writer.write("\n\n")

        response.writer.write("data")
        response.writer.write("\n\n")

        response.writer.write("data:end")
        response.writer.write("\n\n")

        response.writer.flush()
        time.sleep(2)
