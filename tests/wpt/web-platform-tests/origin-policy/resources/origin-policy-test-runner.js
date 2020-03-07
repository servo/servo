window.runTestsInSubframe = ({ hostname, testJS, expectedIds }) => {
  test(() => {
    assert_equals(location.protocol, "https:");
  }, "Prerequisite check: running on HTTPS");

  promise_test(() => new Promise((resolve, reject) => {
    const url = new URL(window.location.href);
    url.hostname = `${hostname}.${document.domain}`;
    url.pathname = "/origin-policy/resources/subframe-with-origin-policy.py";

    // Normalize the URL so that callers can idiomatically give values relative
    // to themselves.
    url.searchParams.append("test", new URL(testJS, document.baseURI).pathname);

    url.searchParams.append("expectedIds", JSON.stringify(expectedIds));

    const iframe = document.createElement("iframe");
    iframe.src = url.href;

    // We need to delegate anything we plan to toggle with FP otherwise it will
    // be locked to disallowed.
    iframe.allow = "camera *; geolocation *";

    iframe.onload = resolve;
    iframe.onerror = () => reject(new Error(`Could not load ${url.href}`));
    document.body.append(iframe);

    fetch_tests_from_window(iframe.contentWindow);
  }), "Test setup of the iframe");
};
