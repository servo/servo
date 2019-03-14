import urllib
import sys, os
sys.path.append(os.path.join(os.path.dirname(__file__), "../common/"))
import sleep

def main(request, response):
    index = request.request_path.index("?")
    args = request.request_path[index+1:].split("&")
    headers = []
    statusSent = False
    headersSent = False
    for arg in args:
        if arg.startswith("ignored"):
            continue
        elif arg.endswith("ms"):
            sleep.sleep_at_least(float(arg[0:-2]))
        elif arg.startswith("redirect:"):
            return (302, "WEBPERF MARKETING"), [("Location", urllib.unquote(arg[9:]))], "TEST"
        elif arg.startswith("mime:"):
            headers.append(("Content-Type", urllib.unquote(arg[5:])))
        elif arg.startswith("send:"):
            text = urllib.unquote(arg[5:])

            if not statusSent:
                # Default to a 200 status code.
                response.writer.write_status(200)
                statusSent = True
            if not headersSent:
                for key, value in headers:
                    response.writer.write_header(key, value)
                response.writer.end_headers()
                headersSent = True

            response.writer.write_content(text)
        elif arg.startswith("status:"):
            code = int(urllib.unquote(arg[7:]))
            response.writer.write_status(code)
            if code // 100 == 1:
                # Terminate informational 1XX responses with an empty line.
                response.writer.end_headers()
            else:
                statusSent = True
        elif arg == "flush":
            response.writer.flush()

#        else:
#            error "  INVALID ARGUMENT %s" % arg

