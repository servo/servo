// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const session_storage_key = "coep_credentialless_iframe_session_storage";
const session_storage_same_origin = "same_origin";
const session_storage_cross_origin = "cross_origin";

promise_test_parallel(async test => {
  // Add an item in the sessionStorage on same_origin.
  sessionStorage.setItem(session_storage_key, session_storage_same_origin);

  // Add an item in the sessionStorage on cross_origin.
  {
    const w_token = token();
    const w_url = cross_origin + executor_path + `&uuid=${w_token}`;
    const w = window.open(w_url);
    const reply_token = token();
    send(w_token, `
      sessionStorage.setItem("${session_storage_key}",
                           "${session_storage_cross_origin}");
      send("${reply_token}", "done");
    `);
    assert_equals(await receive(reply_token), "done");
    w.close();
  }

  promise_test_parallel(async test => {
    let iframe = newAnonymousIframe(same_origin);
    let reply_token = token();
    send(iframe, `
      let value = sessionStorage.getItem("${session_storage_key}");
      send("${reply_token}", value);
    `)
    assert_equals(await receive(reply_token), "")
  }, "same_origin anonymous iframe can't access the sessionStorage");

  promise_test_parallel(async test => {
    let iframe = newAnonymousIframe(cross_origin);
    let reply_token = token();
    send(iframe, `
      let value = sessionStorage.getItem("${session_storage_key}");
      send("${reply_token}", value);
    `)
    assert_equals(await receive(reply_token), "")
  }, "cross_origin anonymous iframe can't access the sessionStorage");

}, "Setup")
