from wptserve.utils import isomorphic_decode
from wptserve.utils import isomorphic_encode
from urllib.parse import unquote

def unescape_query_value(query_value_bytes):
    return isomorphic_encode(unquote(isomorphic_decode(query_value_bytes)))

def main(request, response):
    writable_header = request.headers.get(
        b"Sec-Shared-Storage-Writable",
        b"NO_SHARED_STORAGE_WRITABLE_HEADER")
    response.headers.append(b"Content-Type", b"text/html")
    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Supports-Loading-Mode", b"fenced-frame")
    if writable_header == b"?1" and b'write' in request.GET:
        write_header = unescape_query_value(request.GET[b'write'])
        response.headers.append(b"Shared-Storage-Write", write_header)
    response.content = b'''
<script>
  let parentOrOpener = window.opener || window.parent;
  parentOrOpener.postMessage({ sharedStorageWritableHeader: '%s'}, "*");
</script>
''' % (writable_header)
