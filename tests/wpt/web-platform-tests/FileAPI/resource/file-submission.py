def fail(msg):
    return ([("Content-Type", "text/plain")], "FAIL: " + msg)

def main(request, response):
    content_type = request.headers.get('Content-Type').split("; ")

    if len(content_type) != 2:
        return fail("content type length is incorrect")

    if content_type[0] != 'multipart/form-data':
        return fail("content type first field is incorrect")

    boundary = content_type[1].strip("boundary=")

    body = "--" + boundary + "\r\nContent-Disposition: form-data; name=\"file-input\"; filename=\"upload.txt\""
    body += "\r\n" + "text/plain\r\n\r\nHello\r\n--" + boundary + "--"

    if body != request.body:
        return fail("request body doesn't match: " + body + "+++++++" + request.body)

    return ([("Content-Type", "text/plain")], "OK")
