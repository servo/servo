import time
def main(request, response):
    time.sleep(1)
    headers = [("Content-Type", "text/html")]
    return headers, '''
<!DOCTYPE html>
<head>
</head>
<body>
    DELAYED FRAME
</body
'''
