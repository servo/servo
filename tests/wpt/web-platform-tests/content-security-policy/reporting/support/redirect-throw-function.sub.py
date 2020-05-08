import re

def main(request, response):
    response.status = 302;
    location = re.sub('redirect-throw-function.*',
                      'throw-function.js?secret=1234#ref',
                      request.url)
    response.headers.set("Location", location);
