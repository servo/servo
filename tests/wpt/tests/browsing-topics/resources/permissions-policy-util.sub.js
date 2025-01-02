const TOPICS_PERMISSIONS_POLICY_ERROR_MESSAGE = 'The \"browsing-topics\" Permissions Policy denied the use of document.browsingTopics().';

function test_topics_feature_availability_in_subframe(test, is_same_origin, expect_feature_available_func) {
  const same_origin_src = '/browsing-topics/resources/document-api-notify-parent.tentative.https.html';
  const cross_origin_src = 'https://{{domains[www]}}:{{ports[https][0]}}' +
    same_origin_src;

  let frame = document.createElement('iframe');
  frame.src = is_same_origin ? same_origin_src : cross_origin_src;

  window.addEventListener('message', test.step_func(function handler(evt) {
    if (evt.source === frame.contentWindow) {
      expect_feature_available_func(evt.data);

      document.body.removeChild(frame);
      window.removeEventListener('message', handler);
      test.done();
    }
  }));

  document.body.appendChild(frame);
}

function expect_topics_feature_unavailable(data) {
  assert_equals(data.error, TOPICS_PERMISSIONS_POLICY_ERROR_MESSAGE);
}

function expect_topics_feature_available(data) {
  assert_equals(data.error, 'No error');
}
