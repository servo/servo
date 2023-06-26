// Delay after fetch start:
// - 0.0 seconds: before BFCache
// - 2.0 seconds: when in BFCache
// - 3.5 seconds: after restored from BFCache
function runTest(urlToFetch, hasCSP, shouldSucceed, description) {
  runBfcacheTest({
    funcBeforeNavigation: async (urlToFetch, hasCSP) => {
      if (hasCSP) {
        // Set CSP.
        const meta = document.createElement('meta');
        meta.setAttribute('http-equiv', 'Content-Security-Policy');
        meta.setAttribute('content', "connect-src 'self'");
        document.head.appendChild(meta);
      }

      // Initiate a `fetch()`.
      window.fetchPromise = fetch(urlToFetch);

      // Wait for 0.5 seconds to receive response headers for the fetch()
      // before BFCache, if any.
      await new Promise(resolve => setTimeout(resolve, 500));
    },
    argsBeforeNavigation: [urlToFetch, hasCSP],
    funcBeforeBackNavigation: () => {
      // Wait for 2 seconds before back navigating to pageA.
      return new Promise(resolve => setTimeout(resolve, 2000));
    },
    funcAfterAssertion: async (pageA, pageB, t) => {
      // Wait for fetch() completion and check the result.
      const result = pageA.execute_script(
          () => window.fetchPromise.then(r => r.text()));
      if (shouldSucceed) {
        assert_equals(
          await result,
          'Body',
          'Fetch should complete successfully after restored from BFCache');
      } else {
        await promise_rejects_js(t, TypeError, result,
          'Fetch should fail after restored from BFCache');
      }
    }
  }, 'Eligibility (in-flight fetch): ' + description);
}

const url = new URL('../resources/slow.py', location);
const sameOriginUrl = url.href;
const crossSiteUrl = originCrossSite + url.pathname;
