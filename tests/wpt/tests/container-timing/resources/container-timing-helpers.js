function checkContainerEntry(entry, identifier, last_element_id, beforeRender) {
  assert_equals(entry.entryType, 'container');
  assert_equals(entry.name, 'container-paints');
  assert_equals(entry.identifier, identifier, 'identifier does not match');
  if (last_element_id != null) {
    assert_equals(entry.lastPaintedElement.id, last_element_id);
  }

  assert_equals(entry.duration, 0, 'duration should be 0');
  assert_greater_than_equal(
      entry.firstRenderTime, beforeRender,
      'firstRenderTime greater than beforeRender');
  assert_greater_than_equal(
      entry.startTime, entry.firstRenderTime,
      'startTime greater than beforeRender');
  assert_greater_than_equal(
      performance.now(), entry.startTime, 'startTime bound by now()')

  // PaintTimingMixin
  if ("presentationTime" in entry && entry.presentationTime !== null) {
    assert_greater_than(entry.presentationTime, entry.paintTime);
    assert_equals(entry.presentationTime, entry.startTime);
  } else {
    assert_equals(entry.startTime, entry.paintTime);
  }
}

function checkContainerSize(entry, size) {
  assert_equals(entry.size, size);
}

function onElementTimingEvent(func) {
  const finish_observer = new PerformanceObserver((entryList) => {
    finish_observer.disconnect();
    requestAnimationFrame(() => { func(); });
  });
  finish_observer.observe({ entryTypes: ['element'] });
}

function finishOnElementTiming(t) {
  onElementTimingEvent(() => { t.done(); });
}

function addPaintingElementTimingAfterDoubleRAF(parent) {
  requestAnimationFrame(() => requestAnimationFrame(() => {
    const finish_img = document.createElement('img');
    finish_img.src = '/container-timing/resources/square100.png';
    finish_img.setAttribute('elementtiming', '');
    parent.appendChild(finish_img);
  }));
}
