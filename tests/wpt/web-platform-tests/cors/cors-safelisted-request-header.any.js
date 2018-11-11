// META: script=support.js?pipe=sub
// META: script=/common/utils.js

// This is based on simple-requests.htm, with modifications to make the code more modern and test
// more esoteric cases of header value parsing.

function safelist(headers, expectPreflight = false) {
  promise_test(async t => {
    const uuid = token(),
          url = CROSSDOMAIN + "resources/preflight.py?token=" + uuid,
          checkURL = "resources/preflight.py?check&token=" + uuid,
          request = () => fetch(url, { method: "POST", headers, body: "data" });
    if (expectPreflight) {
      await promise_rejects(t, TypeError(), request());
    } else {
      const response = await request();
      assert_equals(response.headers.get("content-type"), "text/plain");
      assert_equals(await response.text(), "NO");
    }
    const checkResponse = await fetch(checkURL, { method: "POST", body: "data" });
    assert_equals(await checkResponse.text(), (expectPreflight ? "1" : "0"));
  }, (expectPreflight ? "Preflight" : "No preflight") + " for " + JSON.stringify(headers));
}

[
  ["text /plain", true],
  ["text\t/\tplain", true],
  ["text/plain;"],
  ["text/plain;garbage"],
  ["text/plain;garbage\u0001\u0002", true],
  ["text/plain,", true],
  [",text/plain", true],
  ["text/plain,text/plain", true],
  ["text/plain,x/x", true],
  ["text/plain\u000B", true],
  ["text/plain\u000C", true],
  ["application/www-form-urlencoded", true],
  ["application/x-www-form-urlencoded;\u007F", true],
  ["multipart/form-data"],
  ["multipart/form-data;\"", true]
].forEach(([mimeType, preflight = false]) => {
  safelist({"content-type": mimeType}, preflight);
})
