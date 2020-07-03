def main(req, res):
    return ([
        (b'Cache-Control', b'no-cache, must-revalidate'),
        (b'Pragma', b'no-cache'),
        (b'Content-Type', b'application/javascript')],
      b'echo_output = "%s";\n' % req.GET[b'msg'])
