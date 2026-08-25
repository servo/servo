headers = [
    (b"Access-Control-Allow-Origin", b"*"),
]

def main(request, response):
    if b"set" in request.GET:
        headers.append((b"Strict-Transport-Security", b"max-age=60"))
        return (200, headers, "HSTS max-age set to 60.")

    if b"remove" in request.GET:
        headers.append((b"Strict-Transport-Security", b"max-age=0"))
        return (200, headers, "HSTS max-age set to 0.")
