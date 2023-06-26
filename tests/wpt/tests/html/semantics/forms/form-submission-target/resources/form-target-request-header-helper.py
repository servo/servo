body_template="""
<script>
const channel = new BroadcastChannel('{}');
channel.postMessage('{}', '*');
window.close();
</script>
"""
def main(request, response):
    has_content_type = bool(request.headers.get(b'Content-Type'))
    result = u"OK" if has_content_type else u"FAIL"
    channel_name = request.body.decode('utf-8').split("=")[1];
    body = body_template.format(channel_name, result);
    headers = [(b"Content-Type", b"text/html")]
    return headers, body
