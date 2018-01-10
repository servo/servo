def main(request, response):
    headers = []
    # Sets an ETag header to check the cache revalidation behavior.
    headers.append(("ETag", "abc123"))
    headers.append(("Content-Type", "text/javascript"))
    return headers, "/* empty script */"
