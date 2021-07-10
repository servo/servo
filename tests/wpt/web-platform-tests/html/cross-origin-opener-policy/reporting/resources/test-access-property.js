const directory = "/html/cross-origin-opener-policy/reporting";
const executor_path = directory + "/resources/executor.html?pipe=";
const coep_header = '|header(Cross-Origin-Embedder-Policy,require-corp)';

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

const origin = [
  ["same-origin" , same_origin ],
  ["cross-origin", cross_origin],
];
let escapeComma = url => url.replace(/,/g, '\\,');

let testAccessProperty = (property, op, message) => {
  origin.forEach(([origin_name, origin]) => {
    promise_test(async t => {
      const this_window_token = token();

      // The opener window:
      const opener_token = token();
      const opener_url = get_host_info().HTTP_ORIGIN + executor_path +
        `&uuid=${opener_token}`;

      // The openee window:
      const openee_token = token();
      const openee_report_token = token();
      const openee_report_to = reportToHeaders(openee_report_token);
      const openee_url = origin + executor_path + openee_report_to.header +
        openee_report_to.coopReportOnlySameOriginHeader + coep_header +
        `&uuid=${openee_token}`;

      t.add_cleanup(() => {
        send(opener_token, "window.close()")
        send(openee_token, "window.close()")
      });

      // Open the two windows. Wait for them to be loaded.
      window.open(opener_url);
      send(opener_token, `
        window.openee = window.open('${escapeComma(openee_url)}');
      `);
      send(openee_token, `send("${this_window_token}", "Ready");`);
      assert_equals(await receive(this_window_token), "Ready");

      // 2. Try to access the openee.
      send(opener_token, `(${op})(openee);`);

      // 3. Check a reports is sent to the opener.
      let report = await receiveReport(openee_report_token,
                                       "access-to-coop-page-from-opener");
      assert_equals(report.body.property, property);

    }, `${origin_name} > ${op}`);
  })
};
