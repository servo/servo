import imp
import os

here = os.path.split(os.path.abspath(__file__))[0]

def main(request, response):
    auth = imp.load_source("", os.path.join(here,
                                            "..",
                                            "authentication.py"))
    return auth.main(request, response)
