// META: script=/common/utils.js
// META: script=../resources/utils.js
// META: script=/common/get-host-info.sub.js

promise_test(async () => {
    const url = get_host_info().HTTP_REMOTE_ORIGIN + dirname(location.pathname) + RESOURCES_DIR
        + "preflight-unterminated-body.py?token=" + token();
    const response = await fetch(url, { "mode": "cors", "headers": { "x-preflight-test": "1" } });

    assert_equals(response.status, 200, "Actual request succeeded");
    assert_equals(await response.text(), "actual request", "Body came from the actual request");
}, "CORS preflight response with a body that never terminates does not stall the fetch");
