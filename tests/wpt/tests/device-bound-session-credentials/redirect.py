import json

from wptserve.utils import isomorphic_decode

# Returns a redirect to the query param, for testing behavior across
# redirects.
def main(request, response):
    return (302, [("Location",request.url_parts.query)], "")
