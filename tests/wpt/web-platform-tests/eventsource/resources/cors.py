import os
from wptserve import pipes

def run_other(request, response, path):
    #This is a terrible hack
    environ = {"__file__": path}
    exec(compile(open(path, "r").read(), path, 'exec'), environ, environ)
    rv = environ["main"](request, response)
    return rv

def main(request, response):
    origin = request.GET.first("origin", request.headers["origin"])
    credentials = request.GET.first("credentials", "true")

    response.headers.update([("Access-Control-Allow-Origin", origin),
                             ("Access-Control-Allow-Credentials", credentials)])

    handler = request.GET.first('run')
    if handler in ["status-reconnect",
                   "message",
                   "redirect",
                   "cache-control"]:
        if handler == "cache-control":
            response.headers.set("Content-Type", "text/event-stream")
            rv = open(os.path.join(request.doc_root, "eventsource", "resources", "cache-control.event_stream"), "r").read()
            response.content = rv
            pipes.sub(request, response)
            return
        elif handler == "redirect":
            return run_other(request, response, os.path.join(request.doc_root, "common", "redirect.py"))
        else:
            return run_other(request, response, os.path.join(os.path.split(__file__)[0], handler + ".py"))
    else:
        return
