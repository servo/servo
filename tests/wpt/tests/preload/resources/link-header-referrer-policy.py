def main(request, response):
    response_headers = [(b"Link", b"<%s>;rel=\"preload\";%s;as=\"script\"" %
                        (request.GET.first(b"href", b""),
                         request.GET.first(b"preload-policy", b"")))]
    body = ""
    body_name_list = __file__.split(".")[:-1]
    body_name_list.append("html")
    filename = ".".join(body_name_list)
    with open(filename, 'r+b') as f:
        body = f.readlines()
    return (200, response_headers, body)
