from wptserve.utils import isomorphic_decode

def main(request, response):
    key = request.GET[b'key']
    already_requested = request.server.stash.take(key)

    header = [(b'Content-Type', b'application/javascript')]
    initial_script = u'importScripts("./update-missing-import-scripts-imported-worker.py?key={0}")'.format(isomorphic_decode(key))
    updated_script = u'// removed importScripts()'

    if already_requested is None:
        request.server.stash.put(key, True)
        return header, initial_script

    return header, updated_script
