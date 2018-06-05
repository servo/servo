// META: script=/common/utils.js
// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

function headerNames(headers)
{
    let names = [];
    for (let header of headers)
        names.push(header[0].toLowerCase());
    return names
}

/*
  Check preflight is done
  Control if server allows method and headers and check accordingly
  Check control access headers added by UA (for method and headers)
*/
function corsPreflight(desc, corsUrl, method, allowed, headers, safeHeaders) {
  return promise_test(function(test) {
    var uuid_token = token();
    return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(response) {
      var url = corsUrl + (corsUrl.indexOf("?") === -1 ? "?" : "&");
      var urlParameters = "token=" + uuid_token + "&max_age=0";
      var requestInit = {"mode": "cors", "method": method};
      var requestHeaders = [];
      if (headers)
        requestHeaders.push.apply(requestHeaders, headers);
      if (safeHeaders)
        requestHeaders.push.apply(requestHeaders, safeHeaders);
      requestInit["headers"] = requestHeaders;

      if (allowed) {
        urlParameters += "&allow_methods=" + method + "&control_request_headers";
        if (headers) {
          //Make the server allow the headers
          urlParameters += "&allow_headers=" + headerNames(headers).join("%20%2C");
        }
        return fetch(url + urlParameters, requestInit).then(function(resp) {
          assert_equals(resp.status, 200, "Response's status is 200");
          assert_equals(resp.headers.get("x-did-preflight"), "1", "Preflight request has been made");
          if (headers) {
            var actualHeaders = resp.headers.get("x-control-request-headers").toLowerCase().split(",");
            for (var i in actualHeaders)
              actualHeaders[i] = actualHeaders[i].trim();
            for (var header of headers)
              assert_in_array(header[0].toLowerCase(), actualHeaders, "Preflight asked permission for header: " + header);

            let accessControlAllowHeaders = headerNames(headers).sort().join(",");
            assert_equals(resp.headers.get("x-control-request-headers"), accessControlAllowHeaders, "Access-Control-Allow-Headers value");
            return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token);
          } else {
            assert_equals(resp.headers.get("x-control-request-headers"), null, "Access-Control-Request-Headers should be omitted")
          }
        });
      } else {
        return promise_rejects(test, new TypeError(), fetch(url + urlParameters, requestInit)).then(function(){
          return fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token);
        });
      }
    });
  }, desc);
}

var corsUrl = get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

corsPreflight("CORS [DELETE], server allows", corsUrl, "DELETE", true);
corsPreflight("CORS [DELETE], server refuses", corsUrl, "DELETE", false);
corsPreflight("CORS [PUT], server allows", corsUrl, "PUT", true);
corsPreflight("CORS [PUT], server allows, check preflight has user agent", corsUrl + "?checkUserAgentHeaderInPreflight", "PUT", true);
corsPreflight("CORS [PUT], server refuses", corsUrl, "PUT", false);
corsPreflight("CORS [PATCH], server allows", corsUrl, "PATCH", true);
corsPreflight("CORS [PATCH], server refuses", corsUrl, "PATCH", false);
corsPreflight("CORS [NEW], server allows", corsUrl, "NEW", true);
corsPreflight("CORS [NEW], server refuses", corsUrl, "NEW", false);

corsPreflight("CORS [GET] [x-test-header: allowed], server allows", corsUrl, "GET", true, [["x-test-header1", "allowed"]]);
corsPreflight("CORS [GET] [x-test-header: refused], server refuses", corsUrl, "GET", false, [["x-test-header1", "refused"]]);

var headers = [
    ["x-test-header1", "allowedOrRefused"],
    ["x-test-header2", "allowedOrRefused"],
    ["X-test-header3", "allowedOrRefused"],
    ["x-test-header-b", "allowedOrRefused"],
    ["x-test-header-D", "allowedOrRefused"],
    ["x-test-header-C", "allowedOrRefused"],
    ["x-test-header-a", "allowedOrRefused"],
    ["Content-Type", "allowedOrRefused"],
];
var safeHeaders= [
    ["Accept", "*"],
    ["Accept-Language", "bzh"],
    ["Content-Language", "eu"],
];

corsPreflight("CORS [GET] [several headers], server allows", corsUrl, "GET", true, headers, safeHeaders);
corsPreflight("CORS [GET] [several headers], server refuses", corsUrl, "GET", false, headers, safeHeaders);
corsPreflight("CORS [PUT] [several headers], server allows", corsUrl, "PUT", true, headers, safeHeaders);
corsPreflight("CORS [PUT] [several headers], server refuses", corsUrl, "PUT", false, headers, safeHeaders);

corsPreflight("CORS [PUT] [only safe headers], server allows", corsUrl, "PUT", true, null, safeHeaders);
