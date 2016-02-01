if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

function corsCookies(desc, domain1, domain2, credentialsMode, cookies) {
  var urlSetCookie = "http://" + domain1 + ":{{ports[http][0]}}" + dirname(location.pathname) + RESOURCES_DIR + "top.txt";
  var urlCheckCookies = "http://" + domain2 + ":{{ports[http][0]}}" + dirname(location.pathname) + RESOURCES_DIR + "inspect-headers.py?cors&headers=cookie";
  //enable cors with credentials
  var urlParameters = "?pipe=header(Access-Control-Allow-Origin," + location.origin + ")";
  urlParameters += "|header(Access-Control-Allow-Credentials,true)";

  var urlCleanParameters = "?pipe=header(Access-Control-Allow-Origin," + location.origin + ")";
  urlCleanParameters += "|header(Access-Control-Allow-Credentials,true)";
  if (cookies) {
    urlParameters += "|header(Set-Cookie,";
    urlParameters += cookies.join(",True)|header(Set-Cookie,") +  ",True)";
    urlCleanParameters += "|header(Set-Cookie,";
    urlCleanParameters +=  cookies.join("%3B%20max-age=0,True)|header(Set-Cookie,") +  "%3B%20max-age=0,True)";
  }

  var requestInit = {"credentials": credentialsMode, "mode": "cors"};

  promise_test(function(test){
    return fetch(urlSetCookie + urlParameters, requestInit).then(function(resp) {
      assert_equals(resp.status, 200, "HTTP status is 200");
      //check cookies sent
      return fetch(urlCheckCookies, requestInit);
    }).then(function(resp) {
      assert_equals(resp.status, 200, "HTTP status is 200");
      assert_false(resp.headers.has("Cookie") , "Cookie header is not exposed in response");
      if (credentialsMode === "include" && domain1 === domain2) {
        assert_equals(resp.headers.get("x-request-cookie") , cookies.join("; "), "Request includes cookie(s)");
      }
      else {
        assert_false(resp.headers.has("x-request-cookie") , "Request should have no cookie");
      }
      //clean cookies
      return fetch(urlSetCookie + urlCleanParameters, {"credentials": "include"});
    }).catch(function(e) {
      fetch(urlSetCookie + urlCleanParameters, {"credentials": "include"});
      throw e;
    });
  }, desc);
}

var local = "{{host}}";
var remote = "www.{{host}}";
var remote1 = "www1.{{host}}";

corsCookies("Include mode: 1 cookie", remote, remote, "include", ["a=1"]);
corsCookies("Include mode: local cookies are not sent with remote request", local, remote, "include", ["c=3"]);
corsCookies("Include mode: remote cookies are not sent with local request", remote, local, "include", ["d=4"]);
corsCookies("Include mode: remote cookies are not sent with other remote request", remote, remote1, "include", ["e=5"]);
corsCookies("Same-origin mode: cookies are discarded in cors request", remote, remote, "same-origin", ["f=6"]);
corsCookies("Omit mode: no cookie sent", local, local, "omit", ["g=7"]);

done();
