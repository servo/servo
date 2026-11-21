// META: script=/common/get-host-info.sub.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin); webrtc=block` has been set, with no `report-to` endpoint.
// Declaring an endpoint only controls where reports are uploaded, so a
// ReportingObserver should still receive 2 reports, 1 for each
// RTCPeerConnection created in tryConnect().
promise_test(
    async (t) => {
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
      assert_equals(await tryConnect(), 'blocked');

      await report_promise;

      assert_equals(local_reports.length, 2);
      // Convert Report objects to JSON before comparing them.
      assert_object_equals(local_reports[0].toJSON(),
                           local_reports[1].toJSON());
      assert_equals(local_reports[0]['type'], 'connection-allowlist');
      assert_equals(local_reports[0]['url'], location.href);

      const local_body = local_reports[0]['body'];
      assert_equals(local_body['url'], location.href);
      assert_equals(local_body['connection'], 'webrtc');
      assert_array_equals(
          local_body['allowlist'],
          [get_host_info().HTTPS_ORIGIN]);  // header's (response_origin)
      assert_equals(local_body['disposition'], 'enforce');
    },
    'Test that a WebRTC violation report reaches a ReportingObserver in ' +
        'enforce mode when no reporting endpoint is declared.');
