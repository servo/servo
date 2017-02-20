def main(request, response):
    return [('Content-Type', 'text/html'),
            ('X-Content-Type-Options', 'nosniff')], ""
