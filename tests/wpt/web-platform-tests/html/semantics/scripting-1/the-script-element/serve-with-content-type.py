def main(request, response):
    response.headers.append("Content-Type", "text/javascript")
    try:
        file_name = request.GET.first("fn")
        requested_content_type = request.GET.first("ct")
        content = open(file_name, "rb").read()        
    except:
        response.set_error(400, "Not enough parameters")



    return [("Content-Type", "text/plain")], content
