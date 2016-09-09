if (this.document === undefined) {
  importScripts("/common/get-host-info.sub.js")
  importScripts("/resources/testharness.js");
}

var dataURL = "data:text/plain;base64,cmVzcG9uc2UncyBib2R5";
var body = "response's body";
var contentType = "text/plain";

function redirectDataURL(desc, redirectUrl, mode, isOK) {
    var url = redirectUrl +  "?cors&location=" + encodeURIComponent(dataURL);

    var requestInit = {"mode": mode};

    promise_test(function(test) {
        var promise = fetch(url, requestInit).then(function(response) {
            assert_equals(response.type, "opaque", "Response's type should be opaque");
            assert_equals(response.url, "", "Response URL is empty");
            assert_equals(response.status, 0, "Response's status should be 0");
        });
        return isOK ? promise : promise_rejects(test, new TypeError(), promise);
    }, desc);
}

var redirUrl = get_host_info().HTTP_ORIGIN + "/fetch/api/resources/redirect.py";
var corsRedirUrl = get_host_info().HTTP_REMOTE_ORIGIN + "/fetch/api/resources/redirect.py";

redirectDataURL("Testing data URL loading after same-origin redirection (cors mode)", redirUrl, "cors", false);
redirectDataURL("Testing data URL loading after same-origin redirection (no-cors mode)", redirUrl, "no-cors", true);
redirectDataURL("Testing data URL loading after same-origin redirection (same-origin mode)", redirUrl, "same-origin", false);

redirectDataURL("Testing data URL loading after cross-origin redirection (cors mode)", corsRedirUrl, "cors", false);
redirectDataURL("Testing data URL loading after cross-origin redirection (no-cors mode)", corsRedirUrl, "no-cors", true);

done();
