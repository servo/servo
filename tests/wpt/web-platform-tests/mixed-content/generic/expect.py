import json, os, urllib, urlparse

def redirect(url, response):
    response.add_required_headers = False
    response.writer.write_status(301)
    response.writer.write_header("access-control-allow-origin", "*")
    response.writer.write_header("location", url)
    response.writer.end_headers()
    response.writer.write("")

def create_redirect_url(request, swap_scheme = False):
    parsed = urlparse.urlsplit(request.url)
    destination_netloc = parsed.netloc
    scheme = parsed.scheme

    if swap_scheme:
        scheme = "http" if parsed.scheme == "https" else "https"
        hostname = parsed.netloc.split(':')[0]
        port = request.server.config["ports"][scheme][0]
        destination_netloc = ":".join([hostname, str(port)])

    # Remove "redirection" from query to avoid redirect loops.
    parsed_query = dict(urlparse.parse_qsl(parsed.query))
    assert "redirection" in parsed_query
    del parsed_query["redirection"]

    destination_url = urlparse.urlunsplit(urlparse.SplitResult(
        scheme = scheme,
        netloc = destination_netloc,
        path = parsed.path,
        query = urllib.urlencode(parsed_query),
        fragment = None))

    return destination_url

def main(request, response):
    if "redirection" in request.GET:
        redirection = request.GET["redirection"]
        if redirection == "no-redirect":
            pass
        elif redirection == "keep-scheme-redirect":
            redirect(create_redirect_url(request, swap_scheme=False), response)
            return
        elif redirection == "swap-scheme-redirect":
            redirect(create_redirect_url(request, swap_scheme=True), response)
            return
        else:
            raise ValueError ("Invalid redirect type: %s" % redirection)

    content_type = "text/plain"
    response_data = ""

    if "action" in request.GET:
        action = request.GET["action"]

        if "content_type" in request.GET:
            content_type = request.GET["content_type"]

        key = request.GET["key"]
        stash = request.server.stash
        path = request.GET.get("path", request.url.split('?'))[0]

        if action == "put":
            value = request.GET["value"]
            stash.take(key=key, path=path)
            stash.put(key=key, value=value, path=path)
            response_data = json.dumps({"status": "success", "result": key})
        elif action == "purge":
            value = stash.take(key=key, path=path)
            if content_type == "image/png":
                response_data = open(os.path.join(request.doc_root,
                                                  "images",
                                                  "smiley.png")).read()
            elif content_type == "audio/mpeg":
                response_data = open(os.path.join(request.doc_root,
                                                  "media",
                                                  "sound_5.oga")).read()
            elif content_type == "video/mp4":
                response_data = open(os.path.join(request.doc_root,
                                                  "media",
                                                  "movie_5.mp4")).read()
            elif content_type == "application/javascript":
                response_data = open(os.path.join(request.doc_root,
                                                  "mixed-content",
                                                  "generic",
                                                  "worker.js")).read()
            else:
                response_data = "/* purged */"
        elif action == "take":
            value = stash.take(key=key, path=path)
            if value is None:
                status = "allowed"
            else:
                status = "blocked"
            response_data = json.dumps({"status": status, "result": value})

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header("content-type", content_type)
    response.writer.write_header("cache-control", "no-cache; must-revalidate")
    response.writer.end_headers()
    response.writer.write(response_data)
