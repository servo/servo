import json


# Records the Fetch Metadata headers of every request made for a service worker script,
# so that tests can check the requests made when checking for updates.
def main(request, response):
    token = request.GET.first(b"token")

    def header_value(name):
        value = request.headers.get(name)
        return value.decode("utf-8") if value is not None else None

    with request.server.stash.lock:
        records = request.server.stash.take(token)
        if records is None:
            records = []

        is_get_records = b"get_records" in request.GET
        if not is_get_records:
            records.append({
                u"url": request.url,
                u"service-worker": header_value(b"Service-Worker"),
                u"sec-fetch-dest": header_value(b"Sec-Fetch-Dest"),
                u"sec-fetch-mode": header_value(b"Sec-Fetch-Mode"),
                u"sec-fetch-site": header_value(b"Sec-Fetch-Site"),
            })

        request.server.stash.put(token, records)

    if is_get_records:
        return 200, [(b"Content-Type", b"application/json"), (b"Cache-Control", b"no-store")], json.dumps(records)

    kind = request.GET.first(b"kind", b"classic")
    imported_script_url = u"./soft-update-fetch-metadata.py?token=%s&kind=imported" % request.GET.first(b"importtoken", b"").decode("utf-8")

    if kind == b"imported":
        body = u"// Imported script.\n"
    elif kind == b"classic-with-import":
        body = u"importScripts('%s');\nonfetch = e => { e.request; };\n" % imported_script_url
    elif kind == b"module-with-import":
        body = u"import '%s';\nonfetch = e => { e.request; };\n" % imported_script_url
    else:
        body = u"onfetch = e => { e.request; };\n"

    return 200, [(b"Content-Type", b"text/javascript"), (b"Cache-Control", b"no-store")], body
