def main(request, response):
    return 404, [("Content-Type", "text/html")], 'Some content for the masses.' * 100
