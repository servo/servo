const EMPTY_TOPICS_HEADER = '();p=P0000000000000000000000000000000';

function test_topics_iframe_navigation_header(
    test, has_browsing_topics_attribute, is_same_origin, expect_topics_header_available_func) {
  const same_origin_src = '/browsing-topics/resources/check-topics-request-header-notify-parent.py';
  const cross_origin_src = 'https://{{domains[www]}}:{{ports[https][0]}}' +
    same_origin_src;

  let frame = document.createElement('iframe');
  frame.src = is_same_origin ? same_origin_src : cross_origin_src;

  if (has_browsing_topics_attribute) {
    frame.browsingTopics = true;
  }

  window.addEventListener('message', test.step_func(function handler(evt) {
    if (evt.source === frame.contentWindow) {
      expect_topics_header_available_func(evt.data);

      document.body.removeChild(frame);
      window.removeEventListener('message', handler);
      test.done();
    }
  }));

  document.body.appendChild(frame);
}

function expect_topics_header_unavailable(data) {
  assert_equals(data.topicsHeader, 'NO_TOPICS_HEADER');
}

function expect_topics_header_available(data) {
  // An empty result indicates that the request was eligible for topics.
  // Currently, the web-platform-tests framework does not support actually
  // handling the topics request.
  assert_equals(data.topicsHeader, EMPTY_TOPICS_HEADER);
}