def main(request, response):
    response.status = 302
    if b"location" in request.GET:
        response.headers.set(b"Location", request.GET[b"location"])
    else:
        response.headers.set(b"Location", b"post_message_to_frame_owner.html")
