// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// Test that WebRTC Connection-Allowlist violation reports sanitize document URL
// fragments while preserving report.body.connection as "webrtc".

promise_test(async (t) => {
  history.pushState(null, '', '#secret_token');
  const expected_sanitized_url = location.href.split('#')[0];

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
  });

  assert_equals(await tryConnect(), 'blocked');
  await report_promise;

  assert_equals(local_reports.length, 2);
  assert_equals(local_reports[0]['type'], 'connection-allowlist');
  assert_equals(local_reports[0]['url'], expected_sanitized_url);

  const local_body = local_reports[0]['body'];
  assert_equals(local_body['url'], expected_sanitized_url);
  assert_equals(local_body['connection'], 'webrtc');
  assert_array_equals(
      local_body['allowlist'],
      [get_host_info().HTTPS_ORIGIN]);
  assert_equals(local_body['disposition'], 'enforce');

  const endpoint = '/reporting/resources/report.py';
  const id = '6b1a2380-4965-4f4f-9e66-9b6e5e89d123';
  await wait(5000);
  const remote_reports = await pollReports(endpoint, id);
  assert_equals(remote_reports.length, 2);

  assert_equals(remote_reports[0]['type'], 'connection-allowlist');
  assert_equals(remote_reports[0]['url'], expected_sanitized_url);

  const remote_body = remote_reports[0]['body'];
  assert_equals(remote_body['url'], expected_sanitized_url);
  assert_equals(remote_body['connection'], 'webrtc');
  assert_array_equals(
      remote_body['allowlist'],
      [get_host_info().HTTPS_ORIGIN]);
  assert_equals(remote_body['disposition'], 'enforce');
}, 'Test that WebRTC Connection-Allowlist violation reports sanitize document URL fragments while keeping connection="webrtc".');
