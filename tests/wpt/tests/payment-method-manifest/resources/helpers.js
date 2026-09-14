/**
 * Creates a Payment Method Identifier (PMI) URL pointing to
 * payment-method-identifier.py.
 *
 * @param {string} testId - The unique test run token.
 * @param {Object} [options] - URL configuration options.
 * @param {string} [options.host] - Custom host (default: location.host).
 * @param {string|string[]} [options.link] - Custom Link header(s).
 * @param {number} [options.num_redirects] - Number of redirects in chain.
 * @param {string} [options.redirect_location] - Target URL for redirect.
 * @param {number} [options.status] - HTTP response status code.
 * @returns {string} Fully qualified PMI URL.
 */
function createPaymentMethodIdentifierUrl(testId, options = {}) {
  const host = options.host || location.host;
  const url = new URL(`https://${
      host}/payment-method-manifest/resources/payment-method-identifier.py`);
  url.searchParams.set('id', testId);
  if (options.link !== undefined) {
    const links = Array.isArray(options.link) ? options.link : [options.link];
    links.forEach(l => url.searchParams.append('link', l));
  }
  if (options.num_redirects !== undefined) {
    url.searchParams.set('num_redirects', options.num_redirects);
  }
  if (options.redirect_location !== undefined) {
    url.searchParams.set('redirect_location', options.redirect_location);
  }
  if (options.status !== undefined) {
    url.searchParams.set('status', options.status);
  }
  return url.href;
}

/**
 * Creates a Payment Method Manifest URL pointing to payment-method-manifest.py.
 *
 * @param {string} testId - The unique test run token.
 * @param {Object} [options] - URL configuration options.
 * @param {string} [options.host] - Custom host (default: location.host).
 * @param {string} [options.redirect_location] - Target URL for redirect.
 * @param {number} [options.status] - HTTP response status code.
 * @param {string} [options.body] - Custom response body.
 * @param {string} [options.content_type] - Custom Content-Type header.
 * @returns {string} Fully qualified manifest URL.
 */
function createPaymentMethodManifestUrl(testId, options = {}) {
  const host = options.host || location.host;
  const url = new URL(`https://${
      host}/payment-method-manifest/resources/payment-method-manifest.py`);
  url.searchParams.set('id', testId);
  if (options.redirect_location !== undefined) {
    url.searchParams.set('redirect_location', options.redirect_location);
  }
  if (options.status !== undefined) {
    url.searchParams.set('status', options.status);
  }
  if (options.body !== undefined) {
    url.searchParams.set('body', options.body);
  }
  if (options.content_type !== undefined) {
    url.searchParams.set('content_type', options.content_type);
  }
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
