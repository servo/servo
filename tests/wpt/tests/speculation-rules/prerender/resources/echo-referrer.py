"""A page that echoes the Referrer header value via BroadcastChannel.
"""


def main(request, response):
    referrer = request.headers.get(b"referer")
    uid = request.GET.first(b"uid")

    if referrer is None:
        referrer = b"(none)"

    html = u'''
<html>
<head>
<title>Echo referrer</title>
</head>
<script src="/speculation-rules/prerender/resources/utils.js"></script>
<body>
<script>
const bc = new PrerenderChannel('prerender-channel', '%s');
bc.postMessage({referrer: '%s'});
</script>
</body>
</html>
'''
    return (200, [("Content-Type", b"text/html")],
            html % (uid.decode("utf-8"), referrer.decode("utf-8")))
