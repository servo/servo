import time

def main(request, response):
    time.sleep(1.0);

    return [("Content-type", "text/javascript")], """
var s = document.getElementById('script0');
s.innerText = 't.unreached_func("This should not be evaluated")();';
"""
