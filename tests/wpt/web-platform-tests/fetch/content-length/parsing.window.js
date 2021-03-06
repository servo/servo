promise_test(() => {
  return fetch("resources/content-lengths.json").then(res => res.json()).then(runTests);
}, "Loading JSON…");

function runTests(testUnits) {
  testUnits.forEach(({ input, output }) => {
    promise_test(t => {
      const result = fetch(`resources/content-length.py?length=${encodeURIComponent(input)}`);
      if (output === null) {
        return promise_rejects_js(t, TypeError, result);
      } else {
        return result.then(res => res.text()).then(text => {
          assert_equals(text.length, output);
        });
      }
    }, `Input: ${format_value(input)}. Expected: ${output === null ? "network error" : output}.`);
  });
}
