// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin); webrtc=block; report-to=endpoint` has been set.
promise_test(async (t) => {
  let observer = new ReportingObserver(() => {});
  observer.observe();

  try {
    const configuration = {};
    const peerConnection = new RTCPeerConnection(configuration);
    assert_unreached(
        'In enforce mode, RTCPeerConnection creation should throw.');
  } catch (err) {
    assert_equals(err.name, 'NotAllowedError')
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
  assert_equals(local_body['disposition'], 'enforce');

  // Check server-side receipt of the report.
  const endpoint = '/reporting/resources/report.py';
  const id = 'ea5269e0-d728-4173-87a5-da9e6624f6be';
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
  assert_equals(remote_body['disposition'], 'enforce');
}, 'Test that a WebRTC violation report is queued in enforce mode.');
