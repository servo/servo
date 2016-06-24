import urllib
import time

def main(request, response):
    index = request.request_path.index("?")
    args = request.request_path[index+1:].split("&")
    headersSent = 0
    for arg in args:
        if arg.startswith("ignored"):
            continue
        elif arg.endswith("ms"):
            time.sleep(float(arg[0:-2]) / 1E3);
        elif arg.startswith("redirect:"):
            return (302, "WEBPERF MARKETING"), [("Location", urllib.unquote(arg[9:]))], "TEST"
        elif arg.startswith("mime:"):
            response.headers.set("Content-Type", urllib.unquote(arg[5:]))
        elif arg.startswith("send:"):
            text = urllib.unquote(arg[5:])
            if headersSent == 0:
                response.write_status_headers()
                headersSent = 1

            response.writer.write(text)
#        else:
#            error "  INVALID ARGUMENT %s" % arg

