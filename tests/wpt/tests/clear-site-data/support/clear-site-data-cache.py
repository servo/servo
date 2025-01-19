"""
Loaded in Step 2/4/Optional 6 (/clear-site-data/clear-cache.https.html)
Sending Message for Step 3/5/Optional 7 (/clear-site-data/clear-cache.https.html)
"""
import uuid

def main(request, response):
    # type of response:
    #  - "single_html": Main page html file with a different postMessage uuid on each response
    #  - "json": Json that always responds with a different uuid in a single-element array
    #  - "html_embed_json": Main page html that embeds a cachable version of the above json
    response_type = request.GET.first(b"response")

    cache_helper = request.GET.first(b"cache_helper")

    # force enable caching when present or force disable if not
    cache = b"cache" in request.GET
    clear = None
    if b"clear" in request.GET:
        clear = request.GET.first(b"clear")
    if b"clear_first" in request.GET:
        if request.server.stash.take(cache_helper) is None:
            clear = request.GET.first(b"clear_first")
            request.server.stash.put(cache_helper, ())

    headers = []
    if response_type == b"json":
        headers += [(b"Content-Type", b"application/json")]
    else:
        headers += [(b"Content-Type", b"text/html")]

    if cache:
        headers += [(b"cache-control", b"public, max-age=31536000, immutable")]
    else:
        headers += [(b"cache-control", b"no-store")]

    if clear is not None:
        if clear == b"all":
            headers += [(b"Clear-Site-Data", b'"*"')]
        else:
            headers += [(b"Clear-Site-Data", b'"' + clear + b'"')]

    if response_type == b"single_html":
        # send unique UUID. Cache got cleared when uuids don't match.
        content = f'''
            <script>
                window.opener.postMessage("{uuid.uuid4()}" , "*");
            </script>
            <body>
                {request.url}
            </body>'''
    elif response_type == b"json":
        # send unique UUID. helper for below "html_embed_json"
        content = f'''["{uuid.uuid4()}"]'''
    elif response_type == b"html_embed_json":
        url = request.url_parts.path + "?response=json&cache&cache_helper=" + cache_helper.decode()
        content = f'''
            <script>
                fetch("{url}")
                    .then(response => response.json())
                    .then(uuid => window.opener.postMessage(uuid[0], "*"));
            </script>
            <body>
                {request.url}<br>
                {url}
            </body>'''


    return 200, headers, content
