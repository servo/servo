def main(request, response):
    cookie = request.cookies.get(b'auth_cookie')
    if cookie == None or cookie.value != b'abcdef0123':
        return (401, response.headers, "")
    return (200, response.headers, "")
