importScripts('get-host-info.sub.js');

var response_text = "This load was successfully intercepted.";
var response_script = "postMessage(\"This load was successfully intercepted.\");";

self.onfetch = function(event) {
    var url = event.request.url;
    if (url.indexOf("synthesized-response.txt") != -1) {
        event.respondWith(new Response(response_text));
    } else if (url.indexOf("synthesized-response.js") != -1) {
        event.respondWith(new Response(response_script));
    }
};
