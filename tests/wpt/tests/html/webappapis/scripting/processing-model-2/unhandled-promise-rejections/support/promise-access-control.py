def main(request, response):
    allow = request.GET.first(b"allow", b"false")

    headers = [(b"Content-Type", b"application/javascript")]
    if allow != b"false":
        headers.append((b"Access-Control-Allow-Origin", b"*"))

    body = b"""
    	function handleRejectedPromise(promise) {
    		promise.catch(() => {});
    	}

    	(function() {
    		new Promise(function(resolve, reject) { reject(42); });
    	})();
    """

    return headers, body
