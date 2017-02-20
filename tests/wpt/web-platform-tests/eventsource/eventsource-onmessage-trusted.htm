<!DOCTYPE html>
<title>EventSource message events are trusted</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<link rel="help" href="https://html.spec.whatwg.org/multipage/comms.html#dispatchMessage">
<!-- See also:
- https://github.com/whatwg/html/pull/1935
-->

<script>
"use strict";

async_test(t => {
  const source = new EventSource("resources/message.py");

  source.onmessage = t.step_func_done(e => {
    source.close();
    assert_equals(e.isTrusted, true);
  });
}, "EventSource message events are trusted");
</script>
