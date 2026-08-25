var url = new URL("../support/ping.js", document.baseURI).toString();
if (document.getElementById("foo").hasAttribute("blocked-worker"))
  assert_service_worker_is_blocked(url, document.getElementById("foo").getAttribute("data-desc-fallback"));
else
  assert_service_worker_is_loaded(url, document.getElementById("foo").getAttribute("data-desc-fallback"));