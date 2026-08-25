import time

def main(request, response):
    time.sleep(1)
    return [ ("Content-Type", "text/css")], """
    #text {
      font-size: 4em;
    }
    """
