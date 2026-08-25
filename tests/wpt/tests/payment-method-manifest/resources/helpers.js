/**
 * Creates a Payment Method Identifier (PMI) URL pointing to payment-method-identifier.py.
 *
 * @param {string} testId - The unique test run token.
 * @returns {string} Fully qualified PMI URL.
 */
function createPaymentMethodIdentifierUrl(testId) {
  const url = new URL(`https://${location.host}/payment-method-manifest/resources/payment-method-identifier.py`);
  url.searchParams.set('id', testId);
  return url.href;
}

/**
 * Waits for and retrieves server access logs recorded by manifest-server.py for a given test ID.
 *
 * Since manifest fetching and ingesting is asynchronous, this method allows for
 * a desired count of events to be observed before it will return them. Until it
 * reaches that count, it will retry the call every `interval` millisecond, up
 * to a maximum of `timeout` ms. If it never reaches the required number of
 * events, it will throw an Error.
 *
 * @param {Object} t - The testharness test instance (providing t.step_wait).
 * @param {string} testId - The unique test run token.
 * @param {number} requiredCount - Required number of events (default 2: HEAD for PMI, GET for payment method manifest).
 * @param {number} timeout - Timeout in ms. This is passed to step_wait.
 * @param {number} interval - Polling interval in ms. This is passed to step_wait.
 * @returns {Promise<Array>} Array of logged request objects.
 */
async function waitForServerAccessLogs(t, testId, requiredCount = 2, timeout = 3000, interval = 100) {
  const queryUrl = `/payment-method-manifest/resources/stash-query.py?id=${testId}`;
  let lastLogs = [];

  await t.step_wait(
    async () => {
      const resp = await fetch(queryUrl);
      if (!resp.ok) {
        throw new Error(
          `stash-query.py failed with HTTP status ${resp.status}`
        );
      }
      lastLogs = await resp.json();
      return lastLogs && lastLogs.length >= requiredCount;
    },
    `Waiting for ${requiredCount} server access logs`,
    timeout,
    interval
  );

  return lastLogs;
}
