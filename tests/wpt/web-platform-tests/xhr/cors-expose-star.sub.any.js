// META: script=../fetch/api/resources/utils.js

const url = "http://{{host}}:{{ports[http][1]}}" + dirname(location.pathname) + "resources/top.txt",
      sharedHeaders = "?pipe=header(Access-Control-Expose-Headers,*)|header(Test,X)|header(*,whoa)|"

async_test(function() {
  const headers = "header(Access-Control-Allow-Origin,*)"
  var client = new XMLHttpRequest();
  client.open("GET", url + sharedHeaders + headers);
  client.send();
  client.onreadystatechange = this.step_func(function () {
    if (this.readyState == this.HEADERS_RECEIVED) {
      assert_equals(client.getResponseHeader("test"), "X");
      assert_equals(client.getResponseHeader("set-cookie"), null);
      assert_equals(client.getResponseHeader("*"), "whoa");
      this.done();
    }
  });
}, "Basic Access-Control-Expose-Headers: * support")

async_test(function() {
  const origin = location.origin, // assuming an ASCII origin
        headers = "header(Access-Control-Allow-Origin," + origin + ")|header(Access-Control-Allow-Credentials,true)"
  var client = new XMLHttpRequest();
  client.open("GET", url + sharedHeaders + headers);
  client.withCredentials = true;
  client.send();
  client.onreadystatechange = this.step_func(function () {
    if (this.readyState == this.HEADERS_RECEIVED) {
      assert_equals(client.getResponseHeader("content-type"), "text/plain"); // safelisted
      assert_equals(client.getResponseHeader("test"), null);
      assert_equals(client.getResponseHeader("set-cookie"), null);
      assert_equals(client.getResponseHeader("*"), "whoa");
      this.done();
    }
  });
}, "* for credentialed fetches only matches literally")

async_test(function() {
  const headers =  "header(Access-Control-Allow-Origin,*)|header(Access-Control-Expose-Headers,set-cookie\\,*)"
  var client = new XMLHttpRequest();
  client.open("GET", url + sharedHeaders + headers);
  client.send();
  client.onreadystatechange = this.step_func(function () {
    if (this.readyState == this.HEADERS_RECEIVED) {
      assert_equals(client.getResponseHeader("test"), "X");
      assert_equals(client.getResponseHeader("set-cookie"), null);
      assert_equals(client.getResponseHeader("*"), "whoa");
      this.done();
    }
  });
}, "* can be one of several values")
