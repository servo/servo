importScripts("{{location[server]}}/resources/testharness.js");

async_test(function(t) {
  const observer = new ReportingObserver(t.step_func_done((reports) => {
    const url = new URL(reports[0].url);
    // Remove any query parameters which are not relevant for the comparison of the eventual URL with redirects
    url.search = '';
    assert_equals(url.href, "{{location[server]}}/content-security-policy/reporting-api/support/dedicatedworker-report-url.sub.js");
  }));
  observer.observe();

  const url = new URL("{{location[server]}}/content-security-policy/support/ping.js").toString();
  promise_rejects_js(t, TypeError, fetch(url), "Fetch request should be rejected");
}, "URL in report should point to worker where violation occurs{{GET[test-name]}}");

done();
