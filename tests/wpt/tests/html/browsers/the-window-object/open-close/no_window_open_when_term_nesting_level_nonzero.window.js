test(function() {
  var test_window = window.open('', '', 'height=1,width=1');
  var test_document = test_window.document;

  var frame = test_document.createElement('iframe');
  test_document.body.appendChild(frame);

  frame.contentWindow.onpagehide = function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during pagehide");
  };
  frame.contentDocument.onvisibilitychange = function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during visibilitychange");
  };
  frame.contentWindow.onbeforeunload = function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during beforeunload");
  };
  frame.contentWindow.onunload = function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during unload");
  };

  frame.remove();
}, 'no popups with frame removal');

async_test(function(t) {
  var test_window = window.open('', '', 'height=1,width=1');
  var test_document = test_window.document;

  var frame = test_document.createElement('iframe');
  test_document.body.appendChild(frame);

  frame.contentWindow.onpagehide = t.step_func(function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during pagehide");
  });
  frame.contentDocument.onvisibilitychange = t.step_func(function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during visibilitychange");
  });
  frame.contentWindow.onbeforeunload = t.step_func(function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during beforeunload");
  });
  frame.contentWindow.onunload = t.step_func(function(evt) {
    assert_equals(frame.contentWindow.open('', '', 'height=1,width=1'), null,
      "expected no popup during unload");
  });

  frame.onload = t.step_func_done();

  frame.contentWindow.location.href = "about:blank";
}, 'no popups with frame navigation');

async_test(function(t) {
  var test_window = window.open('', '', 'height=1,width=1');
  var test_document = test_window.document;

  var frame = test_document.createElement('iframe');
  test_document.body.appendChild(frame);

  frame.contentWindow.onpagehide = t.step_func(function(evt) {
    assert_equals(test_window.open('', '', 'height=1,width=1'), null,
      "expected no popup during pagehide");
  });
  frame.contentDocument.onvisibilitychange = t.step_func(function(evt) {
    assert_equals(test_window.open('', '', 'height=1,width=1'), null,
      "expected no popup during visibilitychange");
  });
  frame.contentWindow.onbeforeunload = t.step_func(function(evt) {
    assert_equals(test_window.open('', '', 'height=1,width=1'), null,
      "expected no popup during beforeunload");
  });
  frame.contentWindow.onunload = t.step_func(function(evt) {
    assert_equals(test_window.open('', '', 'height=1,width=1'), null,
      "expected no popup during unload");
  });

  frame.onload = t.step_func_done();

  frame.contentWindow.location.href = "about:blank";
}, 'no popups from synchronously reachable window');

async_test(function(t) {
  var test_window = window.open('', '', 'height=1,width=1');
  var test_document = test_window.document;

  var frame = test_document.createElement('iframe');
  test_document.body.appendChild(frame);

  frame.contentWindow.onpagehide = t.step_func(function(evt) {
    assert_equals(window.open('', '', 'height=1,width=1'), null,
      "expected no popup during pagehide");
  });
  frame.contentDocument.onvisibilitychange = t.step_func(function(evt) {
    assert_equals(window.open('', '', 'height=1,width=1'), null,
      "expected no popup during visibilitychange");
  });
  frame.contentWindow.onbeforeunload = t.step_func(function(evt) {
    assert_equals(window.open('', '', 'height=1,width=1'), null,
      "expected no popup during beforeunload");
  });
  frame.contentWindow.onunload = t.step_func(function(evt) {
    assert_equals(window.open('', '', 'height=1,width=1'), null,
      "expected no popup during unload");
  });

  frame.onload = t.step_func_done();

  frame.contentWindow.location.href = "about:blank";
}, 'no popups from another synchronously reachable window');
