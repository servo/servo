import datetime
import time

epoch = datetime.datetime(1970, 1, 1)

def main(req, res):
    # Artificially delay response time in order to ensure uniqueness of
    # computed value
    time.sleep(0.1)

    now = (datetime.datetime.now() - epoch).total_seconds()

    return ([
        (b'Cache-Control', b'no-cache, must-revalidate'),
        (b'Pragma', b'no-cache'),
        (b'Content-Type', b'application/javascript')],
       u'version = "%s";\n' % now)
