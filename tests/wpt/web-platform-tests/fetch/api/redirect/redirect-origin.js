if (this.document === undefined) {
    importScripts("/common/utils.js");
    importScripts("/resources/testharness.js");
    importScripts("../resources/utils.js");
    importScripts("/common/get-host-info.sub.js");
}

function testOriginAfterRedirection(desc, redirectUrl, redirectLocation, redirectStatus, expectedOrigin) {
    var uuid_token = token();
    var url = redirectUrl;
    var urlParameters = "?token=" + uuid_token + "&max_age=0";
    urlParameters += "&redirect_status=" + redirectStatus;
    urlParameters += "&location=" + encodeURIComponent(redirectLocation);

    var requestInit = {"mode": "cors", "redirect": "follow"};

    promise_test(function(test) {
        return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
            assert_equals(resp.status, 200, "Clean stash response's status is 200");
            return fetch(url + urlParameters, requestInit).then(function(response) {
                assert_equals(response.status, 200, "Inspect header response's status is 200");
                assert_equals(response.headers.get("x-request-origin"), expectedOrigin, "Check origin header");
            });
        });
    }, desc);
}

var redirectUrl = RESOURCES_DIR + "redirect.py";
var corsRedirectUrl = get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "redirect.py";
var locationUrl =  get_host_info().HTTP_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "inspect-headers.py?headers=origin";
var corsLocationUrl =  get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "inspect-headers.py?cors&headers=origin";

for (var code of [301, 302, 303, 307, 308]) {
    testOriginAfterRedirection("Same origin to same origin redirection " + code, redirectUrl, locationUrl, code, null);
    testOriginAfterRedirection("Same origin to other origin redirection " + code, redirectUrl, corsLocationUrl, code, get_host_info().HTTP_ORIGIN);
    testOriginAfterRedirection("Other origin to other origin redirection " + code, corsRedirectUrl, corsLocationUrl, code, get_host_info().HTTP_ORIGIN);
    testOriginAfterRedirection("Other origin to same origin redirection " + code, corsRedirectUrl, locationUrl + "&cors", code, "null");
}

done();
