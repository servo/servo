"""
Loaded in Step 2/4/Optional 6 (/clear-site-data/clear-cache.https.html)
Sending Message for Step 3/5/Optional 7 (/clear-site-data/clear-cache.https.html)
"""
import uuid
import random

def generate_png(width, height, color=(0, 0, 0)):
    import zlib
    import struct
    def chunk(chunk_type, data):
        return (
            struct.pack(">I", len(data)) +
            chunk_type +
            data +
            struct.pack(">I", zlib.crc32(chunk_type + data) & 0xffffffff)
        )

    # PNG signature
    png = b"\x89PNG\r\n\x1a\n"

    # IHDR chunk
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    png += chunk(b'IHDR', ihdr)

    # IDAT chunk: RGB pixels
    row = b'\x00' + bytes(color * width)
    raw = row * height
    idat = zlib.compress(raw)
    png += chunk(b'IDAT', idat)

    # IEND chunk
    png += chunk(b'IEND', b'')
    return png

def main(request, response):
    # type of response:
    # - General purpose: returns random uuid or forwards uuid from iframe
    #   - "single_html": Main page html file with a different postMessage uuid on each response
    # - Pages for simple testing normal http cache:
    #   - "json": Json that always responds with a different uuid in a single-element array
    #   - "html_embed_json": Main page html that embeds a cachable version of the above json
    # - Pages for testing specialized caches
    #   - "image": Image that returns random dimensions up to 1024x1024 each time
    #   - "html_embed_image": Main page html that embeds a cachable version of the above image
    #   - "css": Style sheet with random uuid variable
    #   - "html_embed_css": Main page html that embeds a cachable version of the above css
    #   - "js": Script that returns a different uuid each time
    #   - "html_embed_js": Main page html that embeds a cachable version of the above js file
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
    elif response_type == b"image":
        headers += [(b"Content-Type", b"image/png")]
    elif response_type == b"css":
        headers += [(b"Content-Type", b"text/css")]
    elif response_type == b"js":
        headers += [(b"Content-Type", b"text/javascript")]
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
        iframe = ""
        if b"iframe" in request.GET:
            # forward message from iframe to opener
            iframe_url = request.GET.first(b"iframe").decode()
            content = f'''
                <script>
                    // forward iframe uuid to opener
                    window.addEventListener('message', function(event) {{
                        if(window.opener) {{
                            window.opener.postMessage(event.data, "*");
                        }} else {{
                            window.parent.postMessage(event.data, "*");
                        }}
                        window.close();
                    }});
                </script>
                <br>
                {request.url}<br>
                {iframe_url}<br>
                <iframe src="{iframe_url}"></iframe>
                </body>
            '''
        else:
            # send unique UUID. Cache got cleared when uuids don't match.
            u = uuid.uuid4()
            content = f'''
                <script>
                    if(window.opener) {{
                        window.opener.postMessage("{u}", "*");
                    }} else {{
                        window.parent.postMessage("{u}", "*");
                    }}
                    window.close();
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
                    .then(uuid => {{
                        window.opener.postMessage(uuid[0], "*");
                        window.close();
                    }});
            </script>
            <body>
                {request.url}<br>
                {url}
            </body>'''
    elif response_type == b"image":
        # send uniquely sized images, because that info can be retrived from html and definitly using the image cache
        # helper for below "html_embed_image"
        content = generate_png(random.randint(1, 1024), random.randint(1, 1024))
    elif response_type == b"html_embed_image":
        urls = [request.url_parts.path + "?response=image&cache&cache_helper=" + cache_helper.decode() + "&img=" + str(i) for i in range(2)]
        content = f'''
            <!DOCTYPE html>
            <script>
                addEventListener("load", () => {{
                    let img1 = document.getElementById("randomess1");
                    let img2 = document.getElementById("randomess2");
                    let id = img1.naturalWidth + "x" + img1.naturalHeight;
                    id += "-" + img2.naturalWidth + "x" + img2.naturalHeight
                    window.opener.postMessage(id, "*");
                    window.close();
                }})
            </script>
            <body>
                {request.url}<br>
                <img id="randomess1" src="{urls[0]}"></img><br>
                <img id="randomess2" src="{urls[1]}"></img><br>
            </body>'''
    elif response_type == b"css":
        # send unique UUID. helper for below "html_embed_css"
        content = f'''
        :root {{
            --uuid: "{uuid.uuid4()}"
        }}'''
    elif response_type == b"html_embed_css":
        url = request.url_parts.path + "?response=css&cache&cache_helper=" + cache_helper.decode()
        content = f'''
            <!DOCTYPE html>
            <link rel="stylesheet" href="{url}">
            <script>
                let computed = getComputedStyle(document.documentElement);
                let uuid = computed.getPropertyValue("--uuid").trim().replaceAll('"', '');
                window.opener.postMessage(uuid, "*");
                window.close();
            </script>
            <body>
                {request.url}<br>
                {url}
            </body>'''
    elif response_type == b"js":
        # send unique UUID. helper for below "html_embed_js"
        content = f'''
            window.opener.postMessage("{uuid.uuid4()}", "*");
            window.close();
        '''
    elif response_type == b"html_embed_js":
        url = request.url_parts.path + "?response=js&cache&cache_helper=" + cache_helper.decode()
        content = f'''
            <script src="{url}"></script>
            <body>
                {request.url}<br>
                {url}
            </body>'''

    return 200, headers, content
