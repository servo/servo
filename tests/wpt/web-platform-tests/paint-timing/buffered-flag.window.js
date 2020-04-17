async_test(t => {
  assert_implements(window.PerformancePaintTiming, "Paint Timing isn't supported.");
  // First observer creates second in callback to ensure the entry has been dispatched by the time
  // the second observer begins observing.
  let entries_seen = 0;
  new PerformanceObserver(firstList => {
    entries_seen += firstList.getEntries().length;
    // Abort if we have not yet received both paint entries.
    if (entries_seen < 2)
      return;

    // Second observer requires 'buffered: true' to see the entries.
    let firstPaintSeen = false;
    let firstContentfulPaintSeen = false;
    new PerformanceObserver(list => {
      list.getEntries().forEach(t.step_func(entry => {
        assert_equals(entry.entryType, 'paint');
        if (entry.name === 'first-paint')
          firstPaintSeen = true;
        else if (entry.name === 'first-contentful-paint')
          firstContentfulPaintSeen = true;
        else
          assert_unreached('The observer should only see first paint or first contentful paint!');

        if (firstPaintSeen && firstContentfulPaintSeen)
          t.done();
      }));
    }).observe({'type': 'paint', buffered: true});
  }).observe({'entryTypes': ['paint']});

  // Trigger the first paint entries
  const img = document.createElement("IMG");
  img.src = "resources/circles.png";
  document.body.appendChild(img);
}, "PerformanceObserver with buffered flag sees previous paint entries.");
