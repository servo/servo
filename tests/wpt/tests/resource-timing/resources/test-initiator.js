
const with_timeout_message = async (promise, message, timeout = 1000) => {
  return Promise.race([
    promise,
    new Promise((resolve, reject) => {
      step_timeout(() => {
        reject(new Error(message));
      }, timeout);
    }),
  ]);
}

const observe_entry_no_timeout = entry_name => {
  const entry = new Promise(resolve => {
    new PerformanceObserver((entry_list, observer) => {
      for (const entry of entry_list.getEntries()) {
        if (entry.name.endsWith(entry_name)) {
          resolve(entry);
          observer.disconnect();
          return;
        }
      }
    }).observe({"type": "resource", "buffered": true});
  });
  return entry;
};

// Asserts that, for the given name, there is/will-be a
// PerformanceResourceTiming entry that has the given 'initiatorUrl'. The test
// is labeled according to the given descriptor.
const initiator_url_test = (entry_name, expected_url, descriptor, timeout_msg) => {
  promise_test(async () => {
    const promise = observe_entry_no_timeout(entry_name);
    const entry = await with_timeout_message(promise, timeout_msg, /* timeout = */ 2000);
    assert_equals(entry.initiatorUrl, expected_url);
  }, `The initiator Url for ${descriptor} must be '${expected_url}'`);
};

