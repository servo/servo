from time import sleep
def main(request, response):
  if b"delay" in request.GET:
    delay = int(request.GET[b"delay"])
    sleep(delay)

  if b"stylesNotMatchingEnvironment" in request.GET:
    return u'h1 {color: brown;}'
  else:
    return u'h1 {color: purple;}'
