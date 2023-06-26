# Accoridng to routes.py in the wpt server implementation, POST method is
# handled by a Python script handler which requires this file to return an html.
def main(request, response):
    content = """
    <!DOCTYPE html>
      <html>
        <body>
            <a href="blank_page_green.html">navigate away</a>.
        </body>
      </html>
    """
    return content
