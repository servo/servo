import os

def main(request, response):
    directory = os.path.dirname(__file__)

    try:
        file_name = request.GET.first("fn")
        content_type = request.GET.first("ct")
        content = open(os.path.join(directory, file_name), "rb").read()
    
        response.headers.set("Content-Type", content_type)
        response.content = content
    except:
        response.set_error(400, "Not enough parameters or file not found")

    #return [("Content-Type", content_type)], content
