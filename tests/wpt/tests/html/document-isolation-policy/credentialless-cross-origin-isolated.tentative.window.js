// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

const http  = get_host_info().HTTP_ORIGIN;
const https = get_host_info().HTTPS_ORIGIN;

let crossOriginIsolatedTest = (
    description,
    origin ,
    headers,
    expect_crossOriginIsolated) => {
  promise_test_parallel(async test => {
    const w_token = token();
    const w_url = origin + executor_path + headers + `&uuid=${w_token}`;
    const w = window.open(w_url)
    add_completion_callback(() => w.close());

    const this_token = token();
    send(w_token, `
      if (window.crossOriginIsolated)
        send("${this_token}", "crossOriginIsolated");
      else
        send("${this_token}", "not isolated")
    `);
    assert_equals(await receive(this_token), expect_crossOriginIsolated);
  }, description);
}

crossOriginIsolatedTest("Main crossOriginIsolated case:",
  https,  dip_credentialless, "crossOriginIsolated");

crossOriginIsolatedTest("Missing HTTPS:",
  http,  dip_credentialless, "not isolated");

crossOriginIsolatedTest("Report-only:",
  https, dip_report_only_credentialless, "not isolated");

crossOriginIsolatedTest("Report-only + enforced:",
  https, dip_credentialless +
         dip_report_only_credentialless, "crossOriginIsolated");
