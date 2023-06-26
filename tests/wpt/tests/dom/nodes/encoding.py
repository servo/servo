from html import escape

from wptserve.utils import isomorphic_decode

def main(request, response):
    label = request.GET.first(b'label')
    return u"""<!doctype html><meta charset="%s">""" % escape(isomorphic_decode(label))
