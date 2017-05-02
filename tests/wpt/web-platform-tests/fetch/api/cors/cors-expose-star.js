if (this.document === undefined) {
  importScripts("/resources/testharness.js");
  importScripts("../resources/utils.js");
}

const url = "http://{{host}}:{{ports[http][1]}}" + dirname(location.pathname) + RESOURCES_DIR + "top.txt",
      sharedHeaders = "?pipe=header(Access-Control-Expose-Headers,*)|header(Test,X)|header(Set-Cookie,X)|"

promise_test(() => {
  const headers = "header(Access-Control-Allow-Origin,*)"
  return fetch(url + sharedHeaders + headers).then(resp => {
    assert_equals(resp.status, 200)
    assert_equals(resp.type , "cors")
    assert_equals(resp.headers.get("test"), "X")
    assert_equals(resp.headers.get("set-cookie"), null)
  })
}, "Basic Access-Control-Expose-Headers: * support")

promise_test(() => {
  const origin = location.origin, // assuming an ASCII origin
        headers = "header(Access-Control-Allow-Origin," + origin + ")|header(Access-Control-Allow-Credentials,true)"
  return fetch(url + sharedHeaders + headers, { credentials:"include" }).then(resp => {
    assert_equals(resp.status, 200)
    assert_equals(resp.type , "cors")
    assert_equals(resp.headers.get("content-type"), "text/plain") // safelisted
    assert_equals(resp.headers.get("test"), null)
    assert_equals(resp.headers.get("set-cookie"), null)
  })
}, "Cannot use * for credentialed fetches")

done();
