import os.path

from wptserve.utils import isomorphic_decode

def main(request, response):
    type = request.GET.first(b"type", None)

    if type != None and b"svg" in type:
        filename = u"green-96x96.svg"
    else:
        filename = u"blue96x96.png"

    path = os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"../../../images", filename)
    body = open(path, u"rb").read()

    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"x-content-type-options", b"nosniff")
    response.writer.write_header(b"content-length", len(body))
    if(type != None):
        response.writer.write_header(b"content-type", type)
    response.writer.end_headers()

    response.writer.write(body)
