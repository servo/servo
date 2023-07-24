import imp
import os

from wptserve.utils import isomorphic_decode

here = os.path.dirname(os.path.abspath(isomorphic_decode(__file__)))

def main(request, response):
    auth = imp.load_source(u"", os.path.join(here,
                                             u"..",
                                             u"authentication.py"))
    return auth.main(request, response)
