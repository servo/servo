# This responds with a page reporting a click event to the origin provided in
# the `eligible_origin` query param. The pages loads `num_views` copies of
# record-view.py as images in a clickiness-eligible way; with them reporting
# view events.
def main(request, response):
    eligible_origin = request.GET.get(b"eligible_origin")
    num_views = int(request.GET.get(b"num_views"))
    response.status = (200, b"OK")
    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(
        b"Ad-Auction-Record-Event",
        b"type=\"click\", eligible-origins=(\"%s\")" % eligible_origin)

    result = b"<!DOCTYPE html>"
    img_template = b"<img src=\"record-view.py?i=%d&eligible_origin=%s\"" + \
                   b" attributionsrc>"
    for i in range(0, num_views):
      view = img_template % (i, eligible_origin)
      result = result + view
    return result.decode("utf-8")

