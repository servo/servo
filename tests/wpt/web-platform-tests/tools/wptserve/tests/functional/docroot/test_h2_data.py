def handle_data(frame, request, response):
    response.content = frame.data[::-1]
