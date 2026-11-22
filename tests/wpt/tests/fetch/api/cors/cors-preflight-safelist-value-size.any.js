// META: script=/common/utils.js
// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

const corsURL = get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR + "preflight.py";

// Returns a HeadersInit sequence of |name| entries whose values are each
// individually CORS-safelisted (at most 128 bytes) and whose combined value
// size is exactly |totalSize| bytes.
function safelistedEntries(name, totalSize) {
  const entries = [];
  let remaining = totalSize;
  while (remaining > 0) {
    const length = Math.min(remaining, 128);
    entries.push([name, "a".repeat(length)]);
    remaining -= length;
  }
  return entries;
}

function safelistValueSizeTest(desc, entries, expectedRequestHeaders) {
  promise_test(async test => {
    const uuid_token = token();
    let urlParameters = "?token=" + uuid_token + "&max_age=0&control_request_headers";
    if (expectedRequestHeaders) {
      urlParameters += "&allow_methods=GET&allow_headers=" + expectedRequestHeaders;
    }

    await fetch(RESOURCES_DIR + "clean-stash.py?token=" + uuid_token);
    const response = await fetch(corsURL + urlParameters, { mode: "cors", headers: entries });
    assert_equals(response.status, 200, "Response's status is 200");
    if (expectedRequestHeaders) {
      assert_equals(response.headers.get("x-did-preflight"), "1", "Preflight request has been made");
      assert_equals(response.headers.get("x-control-request-headers"), expectedRequestHeaders,
                    "Access-Control-Request-Headers value");
    } else {
      assert_equals(response.headers.get("x-did-preflight"), "0", "No preflight request has been made");
    }
  }, desc);
}

safelistValueSizeTest("Combined safelisted value size of 1024 bytes does not need a CORS-preflight",
                      safelistedEntries("Accept", 1024), null);
safelistValueSizeTest("Combined safelisted value size above 1024 bytes needs a CORS-preflight",
                      safelistedEntries("Accept", 1025), "accept");
safelistValueSizeTest("Combined Content-Language value size above 1024 bytes needs a CORS-preflight",
                      safelistedEntries("Content-Language", 1200), "content-language");
safelistValueSizeTest("Combined safelisted value size above 1024 bytes across header names needs a CORS-preflight",
                      [...safelistedEntries("Accept", 600), ...safelistedEntries("Content-Language", 600)],
                      "accept,content-language");

promise_test(test => {
  const url = corsURL + "?token=" + token() + "&max_age=0&preflight_status=400";
  return promise_rejects_js(test, TypeError,
                            fetch(url, { mode: "cors", headers: safelistedEntries("Accept", 1200) }));
}, "A denied CORS-preflight prevents the request when the combined safelisted value size is above 1024 bytes");
