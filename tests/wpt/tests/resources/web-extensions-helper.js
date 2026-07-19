// testharness file with WebExtensions utilities

/**
 * Loads the WebExtension at the path specified and runs the tests defined in the extension's resources.
 * Listens to messages sent from the user agent and converts the `browser.test` assertions
 * into testharness.js assertions.
 *
 * @param {string} extensionPath - a path to the extension's resources.
 */

setup({ explicit_done: true })
globalThis.runTestsWithWebExtension = function(extensionPath) {
  let test;
  let installPromise;

  function onTestStartedListener(data) {
    test = async_test(data.testName);
  }

  function onTestFinishedListener(data) {
    test.step(() => {
      let description = data.message ?
          `${data.assertionDescription}. ${data.message}` :
          data.assertionDescription;
      assert_true(data.result, description);
    });

    test.done();

    if (data.remainingTests) {
      // There are still tests to perform, don't do any cleanup yet.
      return;
    }

    // Tests have now completed so cleanup.

    browser.test.onTestStarted.removeListener(onTestStartedListener);
    browser.test.onTestFinished.removeListener(onTestFinishedListener);

    // Uninstall the extension before marking the test suite as done.
    installPromise
        .then((extension_id) => {
          return test_driver.uninstall_web_extension(extension_id);
        })
        .then(() => {
          done();
        });
  }

  installPromise =
      test_driver.install_web_extension({type: 'path', path: extensionPath});

  return installPromise.then(() => {
    // Add the test listeners *after* extension install to ensure all browser's will
    // fire test events successfully.
    browser.test.onTestStarted.addListener(onTestStartedListener);
    browser.test.onTestFinished.addListener(onTestFinishedListener);
  });
}
