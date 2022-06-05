def handle_headers(frame, request, response):
    response.status = 203
    response.headers.update([('test', 'passed')])

def handle_data(frame, request, response):
    response.content.append(frame.data.swapcase())
