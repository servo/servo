from six.moves.urllib.parse import unquote

from wptserve.utils import isomorphic_decode, isomorphic_encode

import sleep

def main(request, response):
    index = isomorphic_encode(request.request_path).index(b"?")
    args = isomorphic_encode(request.request_path[index+1:]).split(b"&")
    headers = []
    statusSent = False
    headersSent = False
    for arg in args:
        if arg.startswith(b"ignored"):
            continue
        elif arg.endswith(b"ms"):
            sleep.sleep_at_least(float(arg[0:-2]))
        elif arg.startswith(b"redirect:"):
            return (302, u"WEBPERF MARKETING"), [(b"Location", unquote(isomorphic_decode(arg[9:])))], u"TEST"

        elif arg.startswith(b"mime:"):
            headers.append((b"Content-Type", unquote(isomorphic_decode(arg[5:]))))

        elif arg.startswith(b"send:"):
            text = unquote(isomorphic_decode(arg[5:]))

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
        elif arg.startswith(b"status:"):
            code = int(unquote(isomorphic_decode(arg[7:])))
            response.writer.write_status(code)
            if code // 100 == 1:
                # Terminate informational 1XX responses with an empty line.
                response.writer.end_headers()
            else:
                statusSent = True
        elif arg == b"flush":
            response.writer.flush()

#        else:
#            error "  INVALID ARGUMENT %s" % arg

