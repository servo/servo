// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// This test shows that preventDefault() on the navigate event can
// prevent a soft navigation, because it ensures that neither the a.href
// (foobar.html in our example) is visited, nor the handler specified in the
// intercept on the navigate event is called.

const link = document.createElement('a');
link.href = 'foobar.html';
link.textContent = 'Click me!';
document.body.appendChild(link);

promise_test(async (t) => {
  let navigateProcessed = false;

  navigation.addEventListener('navigate', (e) => {
    e.intercept({
      async handler() {
        assert_unreached('preventDefault() should prevent the navigation');
      },
    });
    e.preventDefault();
    navigateProcessed = true;
  });

  if (test_driver) {
    test_driver.click(link);
  }

  await t.step_wait(
      () => navigateProcessed, '\'navigate\' event not processed');

  const observer = new PerformanceObserver(() => {
    assert_unreached('Soft navigation should not be triggered');
  });
  observer.observe({type: 'soft-navigation', buffered: true});

  await new Promise((resolve) => {
    t.step_timeout(resolve, 3000);
  }).then(() => {
    observer.disconnect();
  });
  if (document.softNavigations) {
    assert_equals(document.softNavigations, 0, 'Soft Navigation not detected');
  }
  assert_false(location.href.includes('foobar.html'), 'foobar.html not visited');
}, 'Navigation API: Aborted navigate event is not a soft navigation');
