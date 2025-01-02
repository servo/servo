import json
import time
def main(request, response):
    uid = request.GET.first(b"uid")
    name = request.GET.first(b"name")
    time.sleep(0.1)

    messagesByName = []
    if request.method == 'POST':
        with request.server.stash.lock:
            messages = request.server.stash.take(uid) or {}
            if name in messages:
                messagesByName = messages[name]

            messagesByName.append(json.loads(request.body))
            messages[name] = messagesByName
            request.server.stash.put(uid, messages)
        response.status = 204
    else:
        with request.server.stash.lock:
            messages = request.server.stash.take(uid) or {}
            if name in messages:
                messagesByName = messages[name]

            request.server.stash.put(uid, messages)
            response.status = 200
            response.headers['Content-Type'] = 'application/json'
            response.content = json.dumps(messagesByName)
