# Test helper that redirects to the location specified by the "location" parameter.
# For use in testing that redirects are blocked in certain contexts. Response
# includes the "Ad-Auction-Allowed" header, which should make no difference;
# it's present to make sure its absence isn't the reason a redirect was blocked.
def main(request, response):
    response.status = (302, "Found")
    response.headers.set(b"Location", request.GET.first(b"location", None))
    response.headers.set(b"Ad-Auction-Allowed", "true")
