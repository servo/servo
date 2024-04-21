#!/usr/bin/python
import json
import urllib
from pywebsocket3 import msgutil
from wptserve import stash

address, authkey = stash.load_env_config()
stash = stash.Stash("/stash_responder", address=address, authkey=authkey)

def web_socket_do_extra_handshake(request):
    return

def web_socket_transfer_data(request):
    while True:
        line = request.ws_stream.receive_message()
        if line == "echo":
            query = request.unparsed_uri.split('?')[1]
            GET = dict(urllib.parse.parse_qsl(query))

            # TODO(kristijanburnik): This code should be reused from
            # /mixed-content/generic/expect.py or implemented more generally
            # for other tests.
            path = GET.get("path", request.unparsed_uri.split('?')[0])
            key = GET["key"]
            action = GET["action"]

            if action == "put":
              value = GET["value"]
              stash.take(key=key, path=path)
              stash.put(key=key, value=value, path=path)
              response_data = json.dumps({"status": "success", "result": key})
            elif action == "purge":
             value = stash.take(key=key, path=path)
             response_data = json.dumps({"status": "success", "result": value})
            elif action == "take":
              value = stash.take(key=key, path=path)
              if value is None:
                  status = "allowed"
              else:
                  status = "blocked"
              response_data = json.dumps({"status": status, "result": value})

            msgutil.send_message(request, response_data)

            return
