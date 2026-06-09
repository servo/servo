// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
//
// The following tests assume the policy `Connection-Allowlist-Report-Only:
// (response-origin); webrtc=block; report-to=endpoint` has been set.
promise_test(async (t) => {
  let observer = new ReportingObserver(() => {});
  observer.observe();

  try {
    const configuration = {};
    const peerConnection = new RTCPeerConnection(configuration);
  } catch (err) {
    assert_unreached(
        'In report-only mode, RTCPeerConnection should be created successfully.');
  }
  observer.disconnect();

  // Check ReportingObserver receipt of the report.
  const local_reports = observer.takeRecords();
  assert_equals(local_reports.length, 1);
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
  assert_equals(remote_reports.length, 1);
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
