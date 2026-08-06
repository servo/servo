// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// The following tests assume the policy `Connection-Allowlist-Report-Only:
// (response-origin); webrtc=block; report-to=endpoint` has been set. There
// should be 2 reports sent, 1 for each RTCPeerConnection created in
// tryConnect().
promise_test(async (t) => {
  let local_reports = [];
  let report_promise = new Promise((resolve) => {
    let observer = new ReportingObserver((reports) => {
      local_reports = local_reports.concat(reports);
      if (local_reports.length >= 2) {
        observer.disconnect();
        resolve();
      }
    });
    observer.observe();
  })
  assert_equals(await tryConnect(), 'allowed');

  // Check ReportingObserver receipt of the report.
  await report_promise;

  assert_equals(local_reports.length, 2);
  // Convert Report objects to JSON before comparing them.
  assert_object_equals(local_reports[0].toJSON(), local_reports[1].toJSON());
  assert_equals(local_reports[0]['type'], 'connection-allowlist');
  assert_equals(local_reports[0]['url'], location.href);

  const local_body = local_reports[0]['body'];
  assert_equals(local_body['url'], location.href);
  assert_equals(local_body['connection'], 'webrtc');
  assert_array_equals(
      local_body['allowlist'],
      [get_host_info().HTTPS_ORIGIN]);  // header's (response_origin)
  assert_equals(local_body['disposition'], 'report');

  // Check server-side receipt of the report.
  const endpoint = '/reporting/resources/report.py';
  const id = '593e9558-bbec-4f10-9cba-ecb85906246a';
  await wait(5000);
  const remote_reports = await pollReports(endpoint, id);
  assert_equals(remote_reports.length, 2);

  // Normalize the "age" property so we can compare the rest of the reports for
  // equality.
  remote_reports[0]['age'] = 0;
  remote_reports[1]['age'] = 0;
  assert_object_equals(remote_reports[0], remote_reports[1]);
  assert_equals(remote_reports[0]['type'], 'connection-allowlist');
  assert_equals(remote_reports[0]['url'], location.href);

  const remote_body = remote_reports[0]['body'];
  assert_equals(remote_body['url'], location.href);
  assert_equals(remote_body['connection'], 'webrtc');
  assert_array_equals(
      remote_body['allowlist'],
      [get_host_info().HTTPS_ORIGIN]);  // header's (response_origin)
  assert_equals(remote_body['disposition'], 'report');
}, 'Test that a WebRTC violation report is queued in report-only mode.');
