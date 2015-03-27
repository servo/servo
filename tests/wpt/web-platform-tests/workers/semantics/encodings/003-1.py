 # -*- coding: utf-8 -*-

def main(request, response):
    return "PASS" if request.GET.first('x') == 'å' else "FAIL"
