var url = new URL("../support/ping.js", document.baseURI).toString();
assert_service_worker_is_loaded(url, document.getElementById("foo").getAttribute("data-desc-fallback"));