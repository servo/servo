if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
  importScripts("/common/utils.js");
}

/* If origin is undefined, it is set to fetched url's origin*/
function corsOrigin(desc, scheme, subdomain, port, method, origin, shouldPass) {
  if (!port)
    port = location.port;
  if (subdomain)
    subdomain = subdomain + ".";
  else
    subdomain = "";
  if (!origin)
    origin = scheme + "://" + subdomain + "{{host}}" + ":" + port;

  var uuid_token = token();
  var urlParameters = "?token=" + uuid_token + "&max_age=0&origin=" + encodeURIComponent(origin) + "&allow_methods=" + method;
  var url = scheme + "://" + subdomain + "{{host}}" + ":" + port + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";
  var requestInit = {"mode": "cors", "method": method};

  promise_test(function(test) {
    fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token).then(function(resp) {
      assert_equals(resp.status, 200, "Clean stash response's status is 200");
      if (shouldPass) {
        return fetch(url + urlParameters, requestInit).then(function(resp) {
          assert_equals(resp.status, 200, "Response's status is 200");
        });
      } else {
        return promise_rejects(test, new TypeError(), fetch(url + urlParameters, requestInit));
      }
    });
  }, desc);

}
var port = "{{ports[http][0]}}";
var port2 = "{{ports[http][1]}}";
var httpsPort = "{{ports[https][0]}}";
/* Actual origin */
var origin = "http://{{host}}:{{ports[http][0]}}";

corsOrigin("Cross domain different subdomain [origin OK]", "http", "www1", undefined, "GET", origin, true);
corsOrigin("Cross domain different subdomain [origin KO]", "http", "www1", undefined, "GET", undefined, false);
corsOrigin("Same domain different port [origin OK]", "http", undefined, port2, "GET", origin, true);
corsOrigin("Same domain different port [origin KO]", "http", undefined, port2, "GET", undefined, false);
corsOrigin("Cross domain different port [origin OK]", "http", "www1", port2, "GET", origin, true);
corsOrigin("Cross domain different port [origin KO]", "http", "www1", port2, "GET", undefined, false);
corsOrigin("Cross domain different protocol [origin OK]", "https", "www1", httpsPort, "GET", origin, true);
corsOrigin("Cross domain different protocol [origin KO]", "https", "www1", httpsPort, "GET", undefined, false);
corsOrigin("Same domain different protocol different port [origin OK]", "https", undefined, httpsPort, "GET", origin, true);
corsOrigin("Same domain different protocol different port [origin KO]", "https", undefined, httpsPort, "GET", undefined, false);
corsOrigin("Cross domain [POST] [origin OK]", "http", "www1", undefined, "POST", origin, true);
corsOrigin("Cross domain [POST] [origin KO]", "http", "www1", undefined, "POST", undefined, false);
corsOrigin("Cross domain [HEAD] [origin OK]", "http", "www1", undefined, "HEAD", origin, true);
corsOrigin("Cross domain [HEAD] [origin KO]", "http", "www1", undefined, "HEAD", undefined, false);
corsOrigin("CORS preflight [PUT] [origin OK]", "http", "www1", undefined, "PUT", origin, true);
corsOrigin("CORS preflight [PUT] [origin KO]", "http", "www1", undefined, "PUT", undefined, false);
corsOrigin("Allowed origin: \"\" [origin KO]", "http", "www1", undefined, "GET", "" , false);

done();
