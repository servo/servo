from wptserve.utils import isomorphic_encode

def main(request, response):
    location = u"%s://%s%s" % (request.url_parts.scheme,
                               request.url_parts.netloc,
                               request.url_parts.path)
    page = u"alternate"
    type = 302
    mix = 0
    if request.GET.first(b"page", None) == b"alternate":
        page = u"default"

    if request.GET.first(b"type", None) == b"301":
        type = 301

    if request.GET.first(b"mix", None) == b"1":
        mix = 1
        type = 302 if type == 301 else 301

    new_location = u"%s?page=%s&type=%s&mix=%s" % (location, page, type, mix)
    headers = [(b"Cache-Control", b"no-cache"),
               (b"Pragma", b"no-cache"),
               (b"Location", isomorphic_encode(new_location))]
    return 301, headers, u"Hello guest. You have been redirected to " + new_location
