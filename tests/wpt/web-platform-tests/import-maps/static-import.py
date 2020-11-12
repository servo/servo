# This file needs to be a sibling of the test files (and not under resources/)
# so that base URL resolution is the same between those test files and <script>s
# pointing to this file.

from wptserve.utils import isomorphic_decode

def main(request, response):
    return (
        ((b'Content-Type', b'text/javascript'),),
        u'import "{}";\n'.format(isomorphic_decode(request.GET.first(b'url')))
    )
