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

    if (!data.result) {
      test.set_status(test.FAIL);
    }

    if (data.remainingTests) {
      return;
    }

    cleanupListeners();
    installPromise
        .then((extension_id) => {
          return test_driver.uninstall_web_extension(extension_id);
        })
        .then(() => {
          done();
        });
  }

  function cleanupListeners() {
    browser.test.onTestStarted.removeListener(onTestStartedListener);
    browser.test.onTestFinished.removeListener(onTestFinishedListener);
  }

  // Attach event listeners synchronously before calling `install_web_extension`
  // to prevent a possible race condition for some browsers where the extension
  // could install and run tests before `installPromise` resolves.
  browser.test.onTestStarted.addListener(onTestStartedListener);
  browser.test.onTestFinished.addListener(onTestFinishedListener);

  installPromise =
      test_driver.install_web_extension({type: 'path', path: extensionPath});

  return installPromise.catch((error) => {
    cleanupListeners();
    throw error;
  });
}
