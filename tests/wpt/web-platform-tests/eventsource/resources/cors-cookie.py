from datetime import datetime
from six import ensure_str

def main(request, response):
    last_event_id = ensure_str(request.headers.get("Last-Event-Id", ""))
    ident = request.GET.first('ident', "test")
    cookie = "COOKIE" if ident in request.cookies else "NO_COOKIE"
    origin = request.GET.first('origin', request.headers["origin"])
    credentials = request.GET.first('credentials', 'true')

    headers = []

    if origin != 'none':
        headers.append(("Access-Control-Allow-Origin", origin));

    if credentials != 'none':
        headers.append(("Access-Control-Allow-Credentials", credentials));

    if last_event_id == '':
        headers.append(("Content-Type", "text/event-stream"))
        response.set_cookie(ident, "COOKIE")
        data = "id: 1\nretry: 200\ndata: first %s\n\n" % cookie
    elif last_event_id == '1':
        headers.append(("Content-Type", "text/event-stream"))
        long_long_time_ago = datetime.now().replace(year=2001, month=7, day=27)
        response.set_cookie(ident, "COOKIE", expires=long_long_time_ago)
        data = "id: 2\ndata: second %s\n\n" % cookie
    else:
        headers.append(("Content-Type", "stop"))
        data = "data: " + last_event_id + cookie + "\n\n";

    return headers, data
