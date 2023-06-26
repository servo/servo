def main(req, res):
    return ([
        (b'Cache-Control', b'no-cache, must-revalidate'),
        (b'Pragma', b'no-cache'),
        (b'Content-Type', b'application/javascript')],
      b'echo_output = "%s (subdir/)";\n' % req.GET[b'msg'])
