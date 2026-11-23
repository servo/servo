import time


def main(request, response):
    mode = request.GET.first(b"mode")

    if mode == b"cross_origin_html":
        fetch_key = request.GET.first(b"fetch_key").decode("utf-8")
        release_key = request.GET.first(b"release_key").decode("utf-8")
        headers = [
            (b"Content-Type", b"text/html; charset=utf-8"),
            (b"Origin-Agent-Cluster", b"?0"),
            (b"Cache-Control", b"no-store"),
        ]
        body = (
            "<!doctype html><meta charset=utf-8>"
            f'<script src="streamed-promise-detach.py?mode=cross_origin_js'
            f'&fetch_key={fetch_key}&release_key={release_key}"></script>'
        )
        return (200, headers, body.encode("utf-8"))

    if mode == b"cross_origin_js":
        fetch_key = request.GET.first(b"fetch_key").decode("utf-8")
        release_key = request.GET.first(b"release_key").decode("utf-8")
        headers = [
            (b"Content-Type", b"text/javascript; charset=utf-8"),
            (b"Cache-Control", b"no-store"),
        ]
        body = (
            "/* padding before fetch call */\n"
            f'fetch("streamed-promise-detach.py?mode=hang_fetch'
            f'&key={fetch_key}&release_key={release_key}").catch(() => {{}});\n'
        )
        return (200, headers, body.encode("utf-8"))

    if mode == b"hang_fetch":
        key = request.GET.first(b"key")
        release_key = request.GET.first(b"release_key")
        request.server.stash.put(key, "in_flight")
        for _ in range(200):
            val = request.server.stash.take(release_key)
            if val is not None:
                break
            time.sleep(0.05)
        return (200, [(b"Content-Type", b"text/plain")], b"done")

    if mode == b"check_fetch":
        key = request.GET.first(b"key")
        headers = [
            (b"Content-Type", b"text/plain"),
            (b"Access-Control-Allow-Origin", b"*"),
            (b"Cache-Control", b"no-store"),
        ]
        val = request.server.stash.take(key)
        if val == "in_flight":
            request.server.stash.put(key, "in_flight")
            return (200, headers, b"in_flight")
        return (200, headers, b"waiting")

    if mode == b"release_fetch":
        release_key = request.GET.first(b"release_key")
        request.server.stash.take(release_key)
        request.server.stash.put(release_key, "done")
        headers = [
            (b"Content-Type", b"text/plain"),
            (b"Access-Control-Allow-Origin", b"*"),
        ]
        return (200, headers, b"ok")

    if mode == b"continue_stream":
        key = request.GET.first(b"key")
        request.server.stash.take(key)
        request.server.stash.put(key, "continue")
        return (200, [(b"Content-Type", b"text/plain")], b"ok")

    if mode == b"stream_html":
        key = request.GET.first(b"key")
        cross_origin_url = request.GET.first(b"cross_origin_url").decode("utf-8")
        response.headers.set(b"Content-Type", b"text/html; charset=utf-8")
        response.headers.set(b"Origin-Agent-Cluster", b"?0")
        response.headers.set(b"Cache-Control", b"no-store")
        response.write_status_headers()

        padding = "<!-- " + ("x" * 4096) + " -->"
        chunk1 = (
            f"<!doctype html>\n<meta charset=utf-8>\n{padding}\n"
            f'<body><b><p><iframe id="cross-origin" src="{cross_origin_url}"></iframe>\n'
        )
        response.writer.write_content(chunk1.encode("utf-8"))

        for _ in range(200):
            val = request.server.stash.take(key)
            if val == "continue":
                break
            time.sleep(0.05)

        chunk2 = b"""</b><script>
  const start = performance.now();
  while (performance.now() - start < 40) {}
  parent.detachReferenceTime = parent.performance.now();
  while (performance.now() - start < 80) {}
</script>"""
        response.writer.write_content(chunk2)
        return
