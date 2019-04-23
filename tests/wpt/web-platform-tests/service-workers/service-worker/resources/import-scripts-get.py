def main(req, res):
    return ([
        ('Cache-Control', 'no-cache, must-revalidate'),
        ('Pragma', 'no-cache'),
        ('Content-Type', 'application/javascript')],
        '%s = "%s";\n' % (req.GET['output'], req.GET['msg']))
