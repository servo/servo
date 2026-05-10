/**
 * @fileoverview Helper class for soft navigation tests.
 *
 * This class provides helper functions for soft navigation tests. It can be
 * used to wait for performance entries and create promises with timeout
 * messages.
 */
class SoftNavigationTestHelper {
  /**
   * Constructs a new instance of the helper class.
   * @param {!Test} test The test object. See
   *     https://web-platform-tests.org/writing-tests/testharness-api.html#test-objects
   */
  constructor(test) {
    this.test_ = test;
  }

  /**
   * Wraps a promise with a timeout message, so that it rejects with this
   * message if it does not resolve within the given timeout.
   * @param {!Promise} promise The promise to wait for.
   * @param {string} message The message to use if the promise times out.
   * @param {number=} timeout The timeout in milliseconds. Defaults to 1000.
   * @return {!Promise} The promise with a timeout message.
   */
  async withTimeoutMessage(promise, message, timeout = 1000) {
    return Promise.race([
      promise,
      new Promise((resolve, reject) => {
        this.test_.step_timeout(() => {
          reject(new Error(message));
        }, timeout);
      }),
    ]);
  }

  /**
   * Creates a new promise with a timeout message, so that it rejects with this
   * message if it does not resolve within the given timeout, and otherwise
   * resolves.
   * @param {!Function} executor The executor function to create the promise;
   *     see
   *     https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/Promise#executor
   * @param {string} message The message to use if the promise times out.
   * @param {number=} timeout The timeout in milliseconds. Defaults to 1000.
   * @return {!Promise} The promise with a timeout message.
   */
  async newPromiseWithTimeoutMessage(executor, message, timeout = 1000) {
    return this.withTimeoutMessage(new Promise(executor), message, timeout);
  }

  /**
   * Waits for a number of buffered performance entries of a given type,
   * optionally including soft navigation observations, with a timeout message.
   * @param {string} type The type of the entries to wait for.
   * @param {number} minNumEntries The minimum number of entries to wait for.
   *     Defaults to 1.
   * @param {number=} timeout The timeout in milliseconds. Defaults to 1000.
   * @return {!Promise} The promise, which either resolves with the entries or
   *     rejects with a timeout message.
   */
  async getBufferedPerformanceEntriesWithTimeout(
      type, minNumEntries = 1, timeout = 1000) {
    let observer;
    return this
        .newPromiseWithTimeoutMessage(
            (resolve) => {
              const entries = [];
              observer = new PerformanceObserver((list) => {
                entries.push(...list.getEntries());
                if (entries.length >= minNumEntries) {
                  resolve(entries);
                }
              })
              observer.observe({
                type: type,
                buffered: true,
              });
            },
            `${minNumEntries} entries of type ${type} never arrived`,
            timeout)
        .finally(() => {
          observer.disconnect();
        });
  }

  /**
   * Waits for a number of performance entries of a given type,
   * optionally including soft navigation observations.
   * @param {string} type The type of the entries to wait for.
   * @param {number} minNumEntries The minimum number of entries to wait for.
   *     Defaults to 1.
   * @return {!Promise} The promise, which resolves with the entries.
   */
  static getPerformanceEntries(type, minNumEntries = 1) {
    return new Promise((resolve) => {
      const entries = [];
      const observer = new PerformanceObserver((list) => {
        entries.push(...list.getEntries());
        if (entries.length >= minNumEntries) {
          resolve(entries);
          observer.disconnect();
        }
      })
      observer.observe({
        type: type,
      });
    });
  }

  /**
   * Clicks on the given click target and validates that a soft navigation
   * occurred and the ICP entry is for the correct element.
   *
   * Note: this only supports a single ICP entry.
   *
   * @param {!HTMLElement} clickTarget The element to click on to navigate.
   * @param {string} url The url to navigate to.
   * @param {function(): (string|Promise<string>)} modifyDOM Function called in
   *    in the click handler to modify the DOM. Can be sync or async. Returns
   *    the ID of the mutated element which is expected to match the ICP entry.
   * @param {function()} navigate Optional function called to navigate the
   *    page. If no function is provided a push navigation to `url` is
   *    performed.
   * @return {!Promise} A promise that is resolved with the resulting soft
   *    navigation and ICP entry.
   */
  async clickAndExpectSoftNavigation(clickTarget, url, modifyDOM, navigate) {
    if (!navigate) {
      navigate = targetUrl => history.pushState({}, '', targetUrl);
    }
    let targetId;
    clickTarget.addEventListener('click', async () => {
      navigate(url);
      targetId = await modifyDOM();
    }, {once: true});

    // Set up the PerformanceObservers before clicking to avoid races.
    const softNavPromise =
        SoftNavigationTestHelper.getPerformanceEntries('soft-navigation');
    const icpPromise =
        SoftNavigationTestHelper.getPerformanceEntries('interaction-contentful-paint');

    if (test_driver) {
      test_driver.click(clickTarget);
    }

    const softNavs = await this.withTimeoutMessage(
        softNavPromise, 'Soft navigation not detected.', /*timeout=*/ 3000);
    assert_equals(softNavs.length, 1, 'Expected exactly one soft navigation.');
    assert_true(
      softNavs[0].name.endsWith(url),
      `Unexpected Soft Navigation URL. Expected url to end with ${url} but got ${softNavs[0].name}`);

    const icps = await this.withTimeoutMessage(
        icpPromise, 'ICP not detected.', /*timeout=*/ 3000);
    assert_equals(icps.length, 1, 'Expected exactly one ICP entry.');
    assert_equals(icps[0].id, targetId, `Expected ICP candidate to be "${targetId}"`);

    return {softNav: softNavs[0], icp: icps[0]};
  }
}
