def main(request, response):
    headers = [('Location', '/echo')]
    return (301, "moved"), headers, ''
