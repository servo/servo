if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("/common/utils.js");
  importScripts("../resources/utils.js");
}

function corsNoPreflight(desc, scheme, subdomain, port, method, headerName, headerValue) {
  if (!port)
    port = location.port;
  if (subdomain)
    subdomain = subdomain + ".";
  else
    subdomain = "";

  var uuid_token = token();
  var url = scheme + "://" + subdomain + "{{host}}" + ":" + port + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";
  var urlParameters = "?token=" + uuid_token + "&max_age=0";
  var requestInit = {"mode": "cors", "method": method, "headers":{}};
  if (headerName)
    requestInit["headers"][headerName] = headerValue;

  promise_test(function(test) {
    fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
      assert_equals(resp.status, 200, "Clean stash response's status is 200");
      return fetch(url + urlParameters, requestInit).then(function(resp) {
        assert_equals(resp.status, 200, "Response's status is 200");
        assert_equals(resp.headers.get("x-did-preflight"), "0", "No preflight request has been made");
      });
    });
  }, desc);
}
var port2 = "{{ports[http][1]}}";
var httpsPort = "{{ports[https][0]}}";

corsNoPreflight("Cross domain basic usage [GET]", "http", "www1", undefined, "GET");
corsNoPreflight("Same domain different port [GET]", "http", undefined, port2, "GET");
corsNoPreflight("Cross domain different port [GET]", "http", "www1", port2, "GET");
corsNoPreflight("Cross domain different protocol [GET]", "https", "www1", httpsPort, "GET");
corsNoPreflight("Same domain different protocol different port [GET]", "https", undefined, httpsPort, "GET");
corsNoPreflight("Cross domain [POST]", "http", "www1", undefined, "POST");
corsNoPreflight("Cross domain [HEAD]", "http", "www1", undefined, "HEAD");
corsNoPreflight("Cross domain [GET] [Accept: */*]", "http", "www1", undefined, "GET" , "Accept", "*/*");
corsNoPreflight("Cross domain [GET] [Accept-Language: fr]", "http", "www1", undefined, "GET" , "Accept-Language", "fr");
corsNoPreflight("Cross domain [GET] [Content-Language: fr]", "http", "www1", undefined, "GET" , "Content-Language", "fr");
corsNoPreflight("Cross domain [GET] [Content-Type: application/x-www-form-urlencoded]", "http", "www1", undefined, "GET" , "Content-Type", "application/x-www-form-urlencoded");
corsNoPreflight("Cross domain [GET] [Content-Type: multipart/form-data]", "http", "www1", undefined, "GET" , "Content-Type", "multipart/form-data");
corsNoPreflight("Cross domain [GET] [Content-Type: text/plain]", "http", "www1", undefined, "GET" , "Content-Type", "text/plain");
corsNoPreflight("Cross domain [GET] [Content-Type: text/plain;charset=utf-8]", "http", "www1", undefined, "GET" , "Content-Type", "text/plain;charset=utf-8");

done();
