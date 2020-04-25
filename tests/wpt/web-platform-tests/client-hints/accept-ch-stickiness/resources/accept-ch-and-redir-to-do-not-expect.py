def main(request, response):
    return 301, [('Location', 'do-not-expect-received.py'),('Accept-CH', 'device-memory, DPR')], ''
