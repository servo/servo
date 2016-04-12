def main(request, response):
    headers = [("Content-Type", "application/javascript")]

    body = {'parse-error': 'var foo = function() {;',
            'undefined-error': 'foo.bar = 42;',
            'uncaught-exception': 'throw new DOMException("AbortError");',
            'caught-exception': 'try { throw new Error; } catch(e) {}',
            'import-malformed-script': 'importScripts("malformed-worker.py?parse-error");',
            'import-no-such-script': 'importScripts("no-such-script.js");'}[request.url_parts.query]
    return headers, body
