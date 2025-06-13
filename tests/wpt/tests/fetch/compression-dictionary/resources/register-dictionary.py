import json

def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    match = b"/fetch/compression-dictionary/resources/*"
    content = b"This is a test dictionary.\n"
    max_age = b"3600"
    if b"match" in request.GET:
        match = request.GET.first(b"match")
    if b"content" in request.GET:
        content = request.GET.first(b"content")
    if b"max-age" in request.GET:
        max_age = request.GET.first(b"max-age")

    token = request.GET.first(b"save_header", None)
    if token is not None:
        headers = {}
        for header in request.headers:
            key = header.decode('utf-8')
            value = request.headers.get(header).decode('utf-8')
            headers[key] = value
        with request.server.stash.lock:
            request.server.stash.put(token, json.dumps(headers))

    previous_token = request.GET.first(b"get_previous_header", None)
    if previous_token is not None:
        result = {}
        with request.server.stash.lock:
            store = request.server.stash.take(previous_token)
            if store is not None:
                headers = json.loads(store)
                result["headers"] = headers
            return json.dumps(result)

    options = b"match=\"" + match + b"\""
    if b"id" in request.GET:
        options += b", id=\"" + request.GET.first(b"id") + b"\""
    response.headers.set(b"Use-As-Dictionary", options)
    response.headers.set(b"Cache-Control", b"max-age=" + max_age)
    if b"age" in request.GET:
        response.headers.set(b"Age", request.GET.first(b"age"))
    response.headers.set(b"Vary", b"available-dictionary,accept-encoding")

    # Compressed responses are generated using the following commands:
    #
    # $ echo "This is a test dictionary." > /tmp/dict
    # $ echo "This is a test dictionary." > /tmp/data
    #
    # $ gzip < /tmp/data > /tmp/out.gz
    # $ xxd -p /tmp/out.gz | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    gzip_data = b"\x1f\x8b\x08\x00\x10\x31\x47\x68\x00\x03\x0b\xc9\xc8\x2c\x56\x00\xa2\x44\x85\x92\xd4\xe2\x12\x85\x94\xcc\xe4\x92\xcc\xfc\xbc\xc4\xa2\x4a\x3d\x2e\x00\x79\xf2\x36\x63\x1b\x00\x00\x00"
    # $ brotli -o /tmp/out.br /tmp/data
    # $ xxd -p /tmp/out.br | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    br_data = b"\xa1\xd0\x00\xc0\x6f\xa4\x74\xf3\x56\xb5\x02\x48\x18\x9d\x2a\x9b\xcb\x42\x14\x81\xa7\x14\xda\x89\x29\x93\x7b\xc8\x09"
    # $ zstd -f -o /tmp/out.zstd /tmp/data
    # $ xxd -p /tmp/out.zstd | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    zstd_data = b"\x28\xb5\x2f\xfd\x24\x1b\xd9\x00\x00\x54\x68\x69\x73\x20\x69\x73\x20\x61\x20\x74\x65\x73\x74\x20\x64\x69\x63\x74\x69\x6f\x6e\x61\x72\x79\x2e\x0a\x3f\x0d\x76\xa0"
    # $ echo -en '\xffDCB' > /tmp/out.dcb
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcb
    # $ brotli --stdout -D /tmp/dict /tmp/data >> /tmp/out.dcb
    # $ xxd -p /tmp/out.dcb | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    dcb_data = b"\xff\x44\x43\x42\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\xa1\xd0\x00\xc0\x2f\x01\x10\xc4\x84\x0a\x05"
    # $ echo -en '\x5e\x2a\x4d\x18\x20\x00\x00\x00' > /tmp/out.dcz
    # $ openssl dgst -sha256 -binary /tmp/dict >> /tmp/out.dcz
    # $ zstd -D /tmp/dict -f -o /tmp/tmp.zstd /tmp/data
    # $ cat /tmp/tmp.zstd >> /tmp/out.dcz
    # $ xxd -p /tmp/out.dcz | tr -d '\n' | sed 's/\(..\)/\\x\1/g'
    dcz_data = b"\x5e\x2a\x4d\x18\x20\x00\x00\x00\x53\x96\x9b\xcf\x5e\x96\x0e\x0e\xdb\xf0\xa4\xbd\xde\x6b\x0b\x3e\x93\x81\xe1\x56\xde\x7f\x5b\x91\xce\x83\x91\x62\x42\x70\xf4\x16\x28\xb5\x2f\xfd\x24\x1b\x35\x00\x00\x00\x01\x00\x1e\x4e\x20\x3f\x0d\x76\xa0"

    if b'content_encoding' in request.GET:
        content_encoding = request.GET.first(b"content_encoding")
        response.headers.set(b"Content-Encoding", content_encoding)
        if content_encoding == b"gzip":
            content = gzip_data
        elif content_encoding == b"br":
            content = br_data
        elif content_encoding == b"zstd":
            content = zstd_data
        elif content_encoding == b"dcb":
            content = dcb_data
        elif content_encoding == b"dcz":
            content = dcz_data

    return content
