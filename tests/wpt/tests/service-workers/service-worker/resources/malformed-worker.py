def main(request, response):
    headers = [(b"Content-Type", b"application/javascript")]

    body = {u'parse-error': u'var foo = function() {;',
            u'undefined-error': u'foo.bar = 42;',
            u'uncaught-exception': u'throw new DOMException("AbortError");',
            u'caught-exception': u'try { throw new Error; } catch(e) {}',
            u'import-malformed-script': u'importScripts("malformed-worker.py?parse-error");',
            u'import-no-such-script': u'importScripts("no-such-script.js");',
            u'top-level-await': u'await Promise.resolve(1);',
            u'instantiation-error': u'import nonexistent from "./imported-module-script.js";',
            u'instantiation-error-and-top-level-await': u'import nonexistent from "./imported-module-script.js"; await Promise.resolve(1);'}[request.url_parts.query]

    return headers, body
