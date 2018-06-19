import os.path

def main(request, response):
    type = request.GET.first("type", None)

    body = open(os.path.join(os.path.dirname(__file__), "green.png"), "rb").read()

    response.add_required_headers = False
    response.writer.write_status(200)

    if 'corp' in request.GET:
        response.writer.write_header("cross-origin-resource-policy", request.GET['corp'])
    if 'acao' in request.GET:
        response.writer.write_header("access-control-allow-origin", request.GET['acao'])
    response.writer.write_header("content-length", len(body))
    if(type != None):
      response.writer.write_header("content-type", type)
    response.writer.end_headers()

    response.writer.write(body)
