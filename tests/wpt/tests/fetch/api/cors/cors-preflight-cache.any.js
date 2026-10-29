// META: script=/common/utils.js
// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

var cors_url = get_host_info().HTTP_REMOTE_ORIGIN +
              dirname(location.pathname) +
              RESOURCES_DIR +
              "preflight.py";

promise_test((test) => {
  var uuid_token = token();
  var request_url =
      cors_url + "?token=" + uuid_token + "&max_age=12000&allow_methods=POST" +
      "&allow_headers=x-test-header";
  return fetch(cors_url + "?token=" + uuid_token + "&clear-stash")
    .then(() => {
      return fetch(
        new Request(request_url,
                    {
                      mode: "cors",
                      method: "POST",
                      headers: [["x-test-header", "test1"]]
                    }));
    })
    .then((resp) => {
      assert_equals(resp.status, 200, "Response's status is 200");
      assert_equals(resp.headers.get("x-did-preflight"), "1", "Preflight request has been made");
      return fetch(cors_url + "?token=" + uuid_token + "&clear-stash");
    })
    .then((res) => res.text())
    .then((txt) => {
      assert_equals(txt, "1", "Server stash must be cleared.");
      return fetch(
        new Request(request_url,
                    {
                      mode: "cors",
                      method: "POST",
                      headers: [["x-test-header", "test2"]]
                    }));
    })
    .then((resp) => {
      assert_equals(resp.status, 200, "Response's status is 200");
      assert_equals(resp.headers.get("x-did-preflight"), "0", "Preflight request has not been made");
      return fetch(cors_url + "?token=" + uuid_token + "&clear-stash");
    });
}, "CORS preflight cache reuses explicit header entries");

// Ref:
//  - https://github.com/whatwg/fetch/issues/1919
//  - https://github.com/whatwg/fetch/issues/1278
promise_test((test) => {
  var uuid_token = token();
  var request_url =
      cors_url + "?token=" + uuid_token + "&max_age=12000&allow_methods=POST" +
      "&allow_headers=*";
  return fetch(cors_url + "?token=" + uuid_token + "&clear-stash")
    .then(() => {
      return fetch(
        new Request(request_url,
                    {
                      mode: "cors",
                      method: "POST",
                      headers: [["x-test-header", "test1"]]
                    }));
    })
    .then((resp) => {
      assert_equals(resp.status, 200, "Response's status is 200");
      assert_equals(resp.headers.get("x-did-preflight"), "1", "Preflight request has been made");
      return fetch(cors_url + "?token=" + uuid_token + "&clear-stash");
    })
    .then((res) => res.text())
    .then((txt) => {
      assert_equals(txt, "1", "Server stash must be cleared.");
      return promise_rejects_js(test, TypeError,
        fetch(
          new Request(request_url,
                      {
                        mode: "cors",
                        method: "POST",
                        headers: [["authorization", "test2"]]
                      })));
    })
    .then(() => fetch(cors_url + "?token=" + uuid_token + "&clear-stash"))
    .then((res) => res.text())
    .then((txt) => {
      assert_equals(txt, "1", "Authorization request made a new preflight");
    });
}, "CORS preflight cache does not reuse wildcard header entries for Authorization");

promise_test(async (test) => {
  const uuid_token = token();
  const request_url =
      cors_url + "?token=" + uuid_token + "&max_age=12000&allow_methods=POST" +
      "&allow_headers=x-test-header";

  await fetch(cors_url + "?token=" + uuid_token + "&clear-stash");

  const resp1 = await fetch(
    new Request(request_url, {
      mode: "cors",
      method: "POST",
      headers: [["x-test-header", "test1"]]
    }));
  assert_equals(resp1.status, 200, "First response status is 200");
  assert_equals(resp1.headers.get("x-did-preflight"), "1", "First request performed preflight");

  const resp2 = await fetch(
    new Request(request_url, {
      mode: "cors",
      method: "POST",
      headers: [["x-test-header", "test2"]]
    }));
  assert_equals(resp2.status, 200, "Second response status is 200");

  const countRes1 = await fetch(cors_url + "?token=" + uuid_token + "&options-count");
  assert_equals(await countRes1.text(), "1", "Only 1 OPTIONS request observed so far");

  await fetch(cors_url + "?token=" + uuid_token + "&fail-next");

  await promise_rejects_js(test, TypeError,
    fetch(
      new Request(request_url, {
        mode: "cors",
        method: "POST",
        headers: [["x-test-header", "test3"]]
      })));

  const countRes2 = await fetch(cors_url + "?token=" + uuid_token + "&options-count");
  assert_equals(await countRes2.text(), "1", "OPTIONS count still 1 after failed request");

  const resp4 = await fetch(
    new Request(request_url, {
      mode: "cors",
      method: "POST",
      headers: [["x-test-header", "test4"]]
    }));
  assert_equals(resp4.status, 200, "Fourth response status is 200");
  assert_equals(resp4.headers.get("x-did-preflight"), "1", "Fourth request performed a new preflight");

  const countRes3 = await fetch(cors_url + "?token=" + uuid_token + "&options-count");
  assert_equals(await countRes3.text(), "2", "2 OPTIONS requests observed in total");
}, "CORS preflight cache entries are cleared when request has network error");

