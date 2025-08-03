import os.path

from wptserve.utils import isomorphic_decode

def main(request, response):
    origin = request.GET[b'origin']
    url = request.GET[b'url']
    expected_referrer = request.GET[b'expected_referrer']
    actual_referrer = request.headers.get(b'referer', b'')

    if expected_referrer == b'none':
        if actual_referrer == b'':
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-green.png"), u"rb").read()
        else:
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-red.png"), u"rb").read()
    elif expected_referrer == b'origin':
        if actual_referrer == origin:
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-green.png"), u"rb").read()
        else:
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-red.png"), u"rb").read()
    elif expected_referrer == b'url':
        if actual_referrer == url:
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-green.png"), u"rb").read()
        else:
            body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-red.png"), u"rb").read()
    else:
        # Return neither red nor green if there is an unexpected "expected_referrer".
        body = open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u"1x1-navy.png"), u"rb").read()

    response.add_required_headers = False
    response.writer.write_status(200)

    if b'corp' in request.GET:
        response.writer.write_header(b"cross-origin-resource-policy", request.GET[b'corp'])
    if b'acao' in request.GET:
        response.writer.write_header(b"access-control-allow-origin", request.GET[b'acao'])
    response.writer.write_header(b"content-length", len(body))
    response.writer.write_header(b"cache-control", b"no-cache; must-revalidate")
    response.writer.end_headers()

    response.writer.write(body)
