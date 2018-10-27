def main(request, response):
    key = request.GET['key']
    already_requested = request.server.stash.take(key)

    header = [('Content-Type', 'application/javascript')]
    initial_script = 'importScripts("./update-missing-import-scripts-imported-worker.py?key={0}")'.format(key)
    updated_script = '// removed importScripts()'

    if already_requested is None:
        request.server.stash.put(key, True)
        return header, initial_script

    return header, updated_script
