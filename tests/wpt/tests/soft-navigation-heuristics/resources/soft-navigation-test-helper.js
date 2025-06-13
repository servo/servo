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
   * @param {boolean} includeSoftNavigationObservations Whether to include
   *     soft navigation observations.
   * @param {number} minNumEntries The minimum number of entries to wait for.
   * @param {number=} timeout The timeout in milliseconds. Defaults to 1000.
   * @return {!Promise} The promise, which either resolves with the entries or
   *     rejects with a timeout message.
   */
  async getBufferedPerformanceEntriesWithTimeout(
      type, includeSoftNavigationObservations, minNumEntries, timeout = 1000) {
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
                includeSoftNavigationObservations:
                    includeSoftNavigationObservations,
              });
            },
            `${minNumEntries} entries of type ${type}${
                includeSoftNavigationObservations ?
                    ' with soft navigation observations' :
                    ''} never arrived`,
            timeout)
        .finally(() => {
          observer.disconnect();
        });
  }

  /**
   * Waits for a number of performance entries of a given type,
   * optionally including soft navigation observations.
   * @param {string} type The type of the entries to wait for.
   * @param {boolean} includeSoftNavigationObservations Whether to include
   *     soft navigation observations.
   * @param {number} minNumEntries The minimum number of entries to wait for.
   * @return {!Promise} The promise, which resolves with the entries.
   */
  static getPerformanceEntries(
      type, includeSoftNavigationObservations, minNumEntries) {
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
        includeSoftNavigationObservations: includeSoftNavigationObservations,
      });
    });
  }
}
