def handle_data(frame, request, response):
    response.content.append(frame.data.swapcase())
