from wptserve.utils import isomorphic_decode
import os

def main(request, response):
    purpose = request.headers.get(b"purpose")
    if (purpose == b'prefetch' and b"code" in request.GET):
        code = int(request.GET.first(b"code"))
    else:
        code = 200

    with open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), "exec.html"), u"r") as fn:
        response.content = fn.read()
    response.status = code
