def main(request, response):
    import datetime, os
    srcpath = os.path.join(os.path.dirname(__file__), "well-formed.xml")
    srcmoddt = datetime.datetime.fromtimestamp(os.path.getmtime(srcpath))
    response.headers.set("Last-Modified", srcmoddt.strftime("%a, %d %b %Y %H:%M:%S GMT"))
    response.headers.set("Content-Type", "application/xml")
    return open(srcpath, "r").read()
