def main(request, response):
    code = int(request.GET.first("code", 200))
    text = request.GET.first("text", "OMG")
    content = request.GET.first("content", "")
    type = request.GET.first("type", "")
    status = (code, text)
    headers = [("Content-Type", type),
               ("X-Request-Method", request.method)]
    return status, headers, content
