def handle_headers(frame, request, response):
    preload_url = request.GET.first(b"preload-url").decode()
    link_header_value = "<{}>; rel=preload; as=script".format(preload_url)

    early_hints = [
        (b":status", b"103"),
        (b"link", link_header_value),
    ]
    response.writer.write_raw_header_frame(headers=early_hints,
                                           end_headers=True)

    redirect_url = request.GET.first(b"redirect-url").decode()
    location = "{}?preload-url={}".format(redirect_url, preload_url)
    response.status = 302
    response.headers["location"] = location
    response.write_status_headers()


def main(request, response):
    pass
