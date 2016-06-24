if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
  importScripts("/common/utils.js");
}

/*
  Check preflight is done
  Control if server allows method and headers and check accordingly
  Check control access headers added by UA (for method and headers)
*/
function corsPreflight(desc, corsUrl, method, allowed, headers) {
  var uuid_token = token();
  fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(response) {

    var url = corsUrl;
    var urlParameters = "?token=" + uuid_token + "&max_age=0";
    var requestInit = {"mode": "cors", "method": method};
    if (headers)
      requestInit["headers"] = headers;

    if (allowed) {
      urlParameters += "&allow_methods=" + method;
      if (headers) {
        //Let's check prefligh request.
        //Server will send back headers from Access-Control-Request-Headers in x-control-request-headers
        urlParameters += "&control_request_headers"
        //Make the server allow the headers
        urlParameters += "&allow_headers="
        urlParameters += headers.map(function (x) { return x[0]; }).join("%2C%20");
      }
      promise_test(function(test) {
        test.add_cleanup(function() {
          fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token);
        });
        return fetch(url + urlParameters, requestInit).then(function(resp) {
          assert_equals(resp.status, 200, "Response's status is 200");
          assert_equals(resp.headers.get("x-did-preflight"), "1", "Preflight request has been made");
          if (headers) {
            var actualHeaders = resp.headers.get("x-control-request-headers").split(",");
            for (var i in actualHeaders)
              actualHeaders[i] = actualHeaders[i].trim();
            for (var header of headers)
              assert_in_array(header[0], actualHeaders, "Preflight asked permission for header: " + header);
          }
        });
      }, desc);
    } else {
      promise_test(function(test) {
        test.add_cleanup(function() {
          fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token);
        });
        return promise_rejects(test, new TypeError(), fetch(url + urlParameters, requestInit));
      }, desc);
    }
  });
}

var corsUrl = "http://www1.{{host}}:{{ports[http][0]}}" + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

corsPreflight("CORS [DELETE], server allows", corsUrl, "DELETE", true);
corsPreflight("CORS [DELETE], server refuses", corsUrl, "DELETE", false);
corsPreflight("CORS [PUT], server allows", corsUrl, "PUT", true);
corsPreflight("CORS [PUT], server refuses", corsUrl, "PUT", false);
corsPreflight("CORS [PATCH], server allows", corsUrl, "PATCH", true);
corsPreflight("CORS [PATCH], server refuses", corsUrl, "PATCH", false);
corsPreflight("CORS [NEW], server allows", corsUrl, "NEW", true);
corsPreflight("CORS [NEW], server refuses", corsUrl, "NEW", false);

corsPreflight("CORS [GET] [x-test-header: allowed], server allows", corsUrl, "GET", true, [["x-test-header1", "allowed"]]);
corsPreflight("CORS [GET] [x-test-header: refused], server refuses", corsUrl, "GET", false, [["x-test-header1", "refused"]]);

var headers = [["x-test-header1", "allowedOrRefused"],
               ["x-test-header2", "allowedOrRefused"],
               ["x-test-header3", "allowedOrRefused"]];
corsPreflight("CORS [GET] [several headers], server allows", corsUrl, "GET", true, headers);
corsPreflight("CORS [GET] [several headers], server refuses", corsUrl, "GET", false, headers);
corsPreflight("CORS [PUT] [several headers], server allows", corsUrl, "PUT", true, headers);
corsPreflight("CORS [PUT] [several headers], server refuses", corsUrl, "PUT", false, headers);
