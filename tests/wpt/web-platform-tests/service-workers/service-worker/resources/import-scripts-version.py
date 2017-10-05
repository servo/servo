import datetime
import time

epoch = datetime.datetime(1970, 1, 1)

def main(req, res):
    # Artificially delay response time in order to ensure uniqueness of
    # computed value
    time.sleep(0.1)

    now = (datetime.datetime.now() - epoch).total_seconds()

    return ([
        ('Cache-Control', 'no-cache, must-revalidate'),
        ('Pragma', 'no-cache'),
        ('Content-Type', 'application/javascript')],
      'version = "%s";\n' % now)
