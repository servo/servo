def main(request, response):
    encoding = request.GET['encoding']
    tmpl = request.GET['tmpl']
    sheet = tmpl % u'\\0000E5'
    return [("Content-Type", "text/css; charset=%s" % encoding)], sheet.encode(encoding)
