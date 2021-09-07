def main(request, response):
    revalidation = 'Cache-Control' in request.headers
    content = request.GET.first(b'content')
    response.headers.set(b'Cache-Control', b'max-age=60')
    response.headers.set(b'ETag', b'assdfsdfe')
    if revalidation:
      response.status = (304, b'NotModified')
    else:
      response.status = (200, b'OK');
      response.write_status_headers()
      response.writer.write(content);
