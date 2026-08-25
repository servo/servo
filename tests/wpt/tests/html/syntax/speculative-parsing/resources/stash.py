def main(request, response):
    if request.GET[b"action"] == b"put":
        encodingcheck = u"param-encodingcheck: " + request.url_parts.query.split(u"&encodingcheck=")[1] + u"\r\n"
        headers = []
        for line in str(request.raw_headers).split(u'\n'):
          header = line.split(':')[0]
          # TODO(zcorpan): also test Cookie
          if header in [u'Origin', u'Accept', u'Referer']:
            headers.append(line)
        request.server.stash.put(request.GET[b"uuid"], encodingcheck + u"\r\n".join(headers))
        return u''
    return request.server.stash.take(request.GET[b"uuid"])
