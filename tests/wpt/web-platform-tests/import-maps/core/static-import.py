from wptserve.utils import isomorphic_decode

def main(request, response):
    return (
        ((b'Content-Type', b'text/javascript'),),
        u'import "{}";\n'.format(isomorphic_decode(request.GET.first(b'url')))
    )
