def main(req, res):
    return ([
        (b'Cache-Control', b'no-cache, must-revalidate'),
        (b'Pragma', b'no-cache'),
        (b'Content-Type', b'application/javascript')],
        b'%s = "%s";\n' % (req.GET[b'output'], req.GET[b'msg']))
