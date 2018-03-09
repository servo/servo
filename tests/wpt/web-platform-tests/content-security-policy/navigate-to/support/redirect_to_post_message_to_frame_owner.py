def main(request, response):
    response.status = 302
    response.headers.set("Location", "post_message_to_frame_owner.html")