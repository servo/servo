import os

from wptserve.utils import isomorphic_decode

from tools.wpt.utils import load_source

here = os.path.dirname(os.path.abspath(isomorphic_decode(__file__)))

def main(request, response):
    auth = load_source(u"", os.path.join(here,
                                             u"..",
                                             u"authentication.py"))
    return auth.main(request, response)
