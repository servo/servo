# Like /common/slow.py except with text/html content-type so that it won't
# trigger strange parts of the <iframe> navigate algorithm.
import time

def main(request, response):
    time.sleep(2)
    return 200, [["Content-Type", "text/html"]], b''
