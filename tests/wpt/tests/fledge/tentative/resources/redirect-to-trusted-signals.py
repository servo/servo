# Test helper that redirects to "trusted-scoring-signals.py" or
# "trusted-bidding-signals.py", depending on whether the query params have an
# "interestGroupNames" entry or not. Query parameters are preserved across the
# redirect. Used to make sure that trusted signals requests don't follow
# redirects.  Response includes the "Ad-Auction-Allowed" header, which should
# make no difference; it's present to make sure its absence isn't the reason a
# redirect was blocked.
def main(request, response):
    response.status = (302, "Found")
    response.headers.set(b"Ad-Auction-Allowed", "true")

    # If there's an "interestGroupNames" query parameter, redirect to bidding
    # signals. Otherwise, redirect to scoring signals.
    location = b"trusted-scoring-signals.py?"
    for param in request.url_parts.query.split("&"):
        pair = param.split("=", 1)
        if pair[0] == "interestGroupNames":
            location = b"trusted-bidding-signals.py?"

    # Append query parameter from current URL to redirect location.
    location += request.url_parts.query.encode("ASCII")
    response.headers.set(b"Location", location)
