setup(_ => {
  assert_implements_optional(
    document.createElement('link').relList.supports('prefetch'),
    "Browser supports prefetch.");
  assert_implements_optional(
    "PerformanceResourceTiming" in window,
    "Browser supports performance APIs.");
});

function assert_resource_not_downloaded(test, url) {
  // CSP failures generate resource timing entries, so let's make sure that
  // download sizes are 0.
  const entries = performance.getEntriesByName(url, 'resource');
  for (const entry of entries) {
    assert_equals(entry.transferSize, 0, 'transferSize');
    assert_equals(entry.encodedBodySize, 0, 'encodedBodySize');
    assert_equals(entry.decodedBodySize, 0, 'decodedBodySize');
  }
}

function assert_link_prefetches(test, link) {
  assert_no_csp_event_for_url(test, link.href);

  link.onerror = test.unreached_func('onerror should not fire.');

  // Test is finished when either the `load` event fires, or we get a performance
  // entry showing that the resource loaded successfully.
  link.onload = test.step_func(test.step_func_done());
  waitUntilResourceDownloaded(link.href).then(test.step_func_done());

  document.head.appendChild(link);
}

function assert_link_does_not_prefetch(test, link) {
  let cspEvent = false;
  let errorEvent = false;

  waitUntilCSPEventForURL(test, link.href)
      .then(test.step_func(e => {
        cspEvent = true;
        assert_equals(e.violatedDirective, "prefetch-src");
        assert_equals(e.effectiveDirective, "prefetch-src");

        if (errorEvent)
          test.done();
      }));

  link.onerror = test.step_func(e => {
    errorEvent = true;
    if (cspEvent)
      test.done();
  });
  link.onload = test.unreached_func('onload should not fire.');

  document.head.appendChild(link);
}

async function try_to_prefetch(href, test) {
  const url = new URL(href, location.href);
  url.searchParams.set(
      'pipe',
      '|header(Cache-Control, max-age=604800)' +
          '|header(Access-Control-Allow-Origin, *)' +
          '|header(Timing-Allow-Origin, *)');
  url.searchParams.set('uuid', token());

  const link = document.createElement('link');
  link.rel = 'prefetch';
  link.href = url.toString();
  link.crossOrigin = 'anonymous';
  test.add_cleanup(() => link.remove());

  const didPrefetch = new Promise(resolve => {
    const observer = new PerformanceObserver(list => {
      const entries = list.getEntriesByName(link.href);
      if (entries.length) {
        resolve(entries[0]);
      }
    });
    observer.observe({entryTypes: ['resource']})
  });
  document.head.appendChild(link);
  const entry = await didPrefetch;
  return entry.requestStart > 0 && entry.decodedBodySize > 0;
}
