 # -*- coding: utf-8 -*-

def main(request, response):
    return u"PASS" if request.GET.first(b'x').decode('utf-8') == u'å' else u"FAIL"
