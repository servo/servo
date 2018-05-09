import helpers

def main(request, response):
    """Respond to `/cookie/imgIfMatch?name={name}&value={value}` with a 404 if
       the cookie isn't present, and a transparent GIF otherwise."""
    headers = helpers.setNoCacheAndCORSHeaders(request, response)
    name = helpers.readParameter(request, paramName="name", requireValue=True)
    value = helpers.readParameter(request, paramName="value", requireValue=True)
    cookiesWithMatchingNames = request.cookies.get_list(name)
    for cookie in cookiesWithMatchingNames:
        if cookie.value == value:
            # From https://github.com/mathiasbynens/small/blob/master/gif-transparent.gif
            headers.append(("Content-Type","image/gif"))
            gif = "\x47\x49\x46\x38\x39\x61\x01\x00\x01\x00\x80\x00\x00\xFF\xFF\xFF\x00\x00\x00\x21\xF9\x04\x01\x00\x00\x00\x00\x2C\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3B"
            return headers, gif
    return 500, headers, '{"error": {"message": "The cookie\'s value did not match the given value."}}'
