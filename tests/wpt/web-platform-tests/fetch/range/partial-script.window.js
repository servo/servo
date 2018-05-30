// META: script=resources/utils.js

// It's weird that browsers do this, but it should continue to work.
promise_test(async t => {
  await loadScript('resources/partial-script.py?pretend-offset=90000');
  assert_true(self.scriptExecuted);
}, `Script executed from partial response`);
