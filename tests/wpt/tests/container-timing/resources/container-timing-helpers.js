function checkContainerEntry(entry, identifier, last_element_id, beforeRender) {
  assert_equals(entry.entryType, 'container');
  assert_equals(entry.name, '');
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

function checkContainerIntersectionRect(entry, expected, description = '') {
  assert_equals(
      entry.intersectionRect.left, expected[0], 'left of rect ' + description);
  assert_equals(
      entry.intersectionRect.right, expected[1],
      'right of rect ' + description);
  assert_equals(
      entry.intersectionRect.top, expected[2], 'top of rect ' + description);
  assert_equals(
      entry.intersectionRect.bottom, expected[3],
      'bottom of rect ' + description);
}

// Identifiers of the container roots added by waitForPendingPaints(). Their
// entries are left out of the ones returned to the tests.
const kWaitIdentifierPrefix = 'container-timing-wait-';
let wait_count = 0;

function isWaitEntry(entry) {
  return entry.identifier.startsWith(kWaitIdentifierPrefix);
}

// Resolves once the paints queued so far have been reported, by adding an
// image that is its own container root and waiting for its entry.
// containertimingignore keeps it out of any ancestor root, e.g. a <body> one.
//
// This observes container entries, so tests that need no container observer,
// or run after Container Timing stops recording, wait with timeouts instead.
function waitForPendingPaints() {
  const identifier = kWaitIdentifierPrefix + wait_count++;
  return new Promise(resolve => {
    const observer = new PerformanceObserver(entryList => {
      if (entryList.getEntries().some(e => e.identifier === identifier)) {
        observer.disconnect();
        img.remove();
        resolve();
      }
    });
    observer.observe({type: 'container'});

    // Pin inside the viewport and out of normal flow so tall/scrolled pages do
    // not clip it offscreen (suppressing its entry), adjacent whitespace inside
    // a <body> root still collapses without painting as text, and layout does
    // not shift.
    const img = addSquare100Image(document.body, {
      containertiming: identifier,
      containertimingignore: '',
      style: [
        'position: fixed',
        'right: 0',
        'bottom: 0',
        'width: 1px',
        'height: 1px',
        'pointer-events: none',
      ].join('; '),
    });
  });
}

// Returns the container entries, buffered ones included, after waiting for at
// least `minNumEntries` of them and then for every other pending paint, so
// tests also catch extra or late entries. Fails if `minNumEntries` have not
// arrived after `timeout` ms.
async function getContainerEntries(t, minNumEntries = 1, timeout = 3000) {
  const entries = [];
  let observer;
  try {
    await new Promise((resolve, reject) => {
      observer = new PerformanceObserver(entryList => {
        entries.push(...entryList.getEntries().filter(e => !isWaitEntry(e)));
        if (entries.length >= minNumEntries) {
          resolve();
        }
      });
      observer.observe({type: 'container', buffered: true});
      if (minNumEntries === 0) {
        resolve();
        return;
      }
      t.step_timeout(() => {
        reject(new Error(`${minNumEntries} container entries never arrived`));
      }, timeout);
    });
    await waitForPendingPaints();
  } finally {
    observer.disconnect();
  }
  return entries;
}

// Returns the container entries reported in the next `duration` ms, without
// buffered ones. For tests where no container entry can end the wait.
async function getContainerEntriesWithin(t, duration = 1000) {
  const entries = [];
  const observer = new PerformanceObserver(entryList => {
    entries.push(...entryList.getEntries());
  });
  observer.observe({type: 'container'});
  await waitWithoutObserving(t, duration);
  observer.disconnect();
  return entries;
}

// Waits `duration` ms, long enough for pending paints to be reported, without
// observing container entries.
function waitWithoutObserving(t, duration = 3000) {
  return new Promise(resolve => t.step_timeout(resolve, duration));
}

// Attributes are set before the element is inserted.
function addElement(parent, tagName, attributes = {}) {
  const element = document.createElement(tagName);
  for (const [name, value] of Object.entries(attributes)) {
    element.setAttribute(name, value);
  }
  parent.appendChild(element);
  return element;
}

function addContainerRoot(parent, identifier, attributes = {}) {
  return addElement(parent, 'div',
                    {containertiming: identifier, ...attributes});
}

function addSquare100Image(parent, attributes = {}) {
  return addElement(
      parent, 'img',
      {src: '/container-timing/resources/square100.png', ...attributes});
}

function addText(parent, attributes = {}) {
  const p = addElement(parent, 'p', attributes);
  p.innerText = 'This is a text';
  return p;
}
