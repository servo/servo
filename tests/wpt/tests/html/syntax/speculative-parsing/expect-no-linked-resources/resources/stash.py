import time

def main(request, response):
    if request.GET[b"action"] == b"put":
        # Received result data from target page
        request.server.stash.put(request.GET[b"uuid"], request.GET[b"uuid"], u'/expect-no-linked-resources/')
        return u"ok"
    else:
        # Request for result data from test page
        value = request.server.stash.take(request.GET[b"uuid"], u'/expect-no-linked-resources/')
        return value
