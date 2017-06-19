def main(req, res):
    return ([
        ('Cache-Control', 'no-cache, must-revalidate'),
        ('Pragma', 'no-cache'),
        ('Content-Type', 'application/javascript')],
      'echo_output = "%s";\n' % req.GET['msg'])
