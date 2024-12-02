import json

SUBDOMAIN_TUPLE_TO_ALLOWED_ORIGINS_MAP = {
    ("", "web-platform"): [],
    ("www", "web-platform"): [
        {
            "scriptOrigin": "https://web-platform.test",
            "contextOrigin": ["https://web-platform.test"],
         },
        {
            "scriptOrigin": "https://www.web-platform.test",
            "contextOrigin": "*",
         },
    ],
    ("www1", "web-platform"): [
        {
            "scriptOrigin": [
                "https://google.com",
                "https://web-platform.test",
            ],
            "contextOrigin": "https://web-platform.test",
         },
        {
            "scriptOrigin": "https://www.web-platform.test",
            "contextOrigin": ["*"],
         },
    ],
    ("www2", "web-platform"): [],
    ("", "not-web-platform"): [],
    ("www", "not-web-platform"): [],
    ("www1", "not-web-platform"): [],
    ("www2", "not-web-platform"): [],
}

def get_host(request):
    return request.url_parts.netloc.split(":")[0]

def get_port(request):
    if len(request.url_parts.netloc.split(":")) > 1:
        return request.url_parts.netloc.split(":")[1]
    return 0

def get_subdomain_tuple(request):
    host = get_host(request)
    host_list = host.split(".")
    if len(host_list) == 0 or len(host_list) > 3:
        raise ValueError("Invalid host " + host)
    if len(host_list) == 1:
        return ("", host_list[0])
    return (host_list[0], host_list[1])

def get_allowed_origins(request):
    subdomain_tuple = get_subdomain_tuple(request)
    origin_list = SUBDOMAIN_TUPLE_TO_ALLOWED_ORIGINS_MAP[subdomain_tuple]
    origins_string = json.dumps(origin_list)
    test_with_port = ".test:" + str(get_port(request))
    origins_with_ports = origins_string.replace(".test", test_with_port)
    return origins_with_ports

def get_json(request, response):
    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"application/json")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.content = get_allowed_origins(request)
