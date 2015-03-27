def main(request, response):
    id = request.GET['id']
    encoding = request.GET['encoding']
    mode = request.GET['mode']
    iframe = ""
    if mode == 'NETWORK':
        iframe = "<iframe src='stash.py?q=%%C3%%A5&id=%s&action=put'></iframe>" % id
    doc = """<!doctype html>
<html manifest="manifest.py?id=%s&encoding=%s&mode=%s">
%s
""" % (id, encoding, mode, iframe)
    return [("Content-Type", "text/html; charset=%s" % encoding)], doc.encode(encoding)
