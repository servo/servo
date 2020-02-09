def main(request, response):
    response.headers.set("Access-Control-Allow-Origin", "*");
    response.headers.set("Access-Control-Max-Age", "0");
    response.headers.set("Timing-Allow-Origin", "*");
    # If this script is accessed with the header X-Require-Preflight then the
    # browser will send a preflight request. Otherwise it won't.
    if request.method == 'OPTIONS':
        response.headers.set("Access-Control-Allow-Headers", "X-Require-Preflight");
