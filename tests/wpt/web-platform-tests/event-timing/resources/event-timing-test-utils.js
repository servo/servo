// Clicks on the element with the given ID. It adds an event handler to the element which
// ensures that the events have a duration of at least |delay|. Calls |callback| during
// event handler if |callback| is provided.
async function clickOnElementAndDelay(id, delay, callback) {
  const element = document.getElementById(id);
  const clickHandler = () => {
    mainThreadBusy(delay);
    if (callback) {
      callback();
    }
    element.removeEventListener("pointerdown", clickHandler);
  };

  element.addEventListener("pointerdown", clickHandler);
  await test_driver.click(element);
}

function mainThreadBusy(duration) {
  const now = performance.now();
  while (performance.now() < now + duration);
}

// This method should receive an entry of type 'event'. |isFirst| is true only when we want
// to check that the event also happens to correspond to the first event. In this case, the
// timings of the 'first-input' entry should be equal to those of this entry. |minDuration|
// is used to compared against entry.duration.
function verifyEvent(entry, eventType, targetId, isFirst=false, minDuration=104, notCancelable=false) {
  assert_equals(entry.cancelable, !notCancelable, 'cancelable property');
  assert_equals(entry.name, eventType);
  assert_equals(entry.entryType, 'event');
  assert_greater_than_equal(entry.duration, minDuration,
      "The entry's duration should be greater than or equal to " + minDuration + " ms.");
  assert_greater_than_equal(entry.processingStart, entry.startTime,
      "The entry's processingStart should be greater than or equal to startTime.");
  assert_greater_than_equal(entry.processingEnd, entry.processingStart,
      "The entry's processingEnd must be at least as large as processingStart.");
  // |duration| is a number rounded to the nearest 8 ms, so add 4 to get a lower bound
  // on the actual duration.
  assert_greater_than_equal(entry.duration + 4, entry.processingEnd - entry.startTime,
      "The entry's duration must be at least as large as processingEnd - startTime.");
  if (isFirst) {
    let firstInputs = performance.getEntriesByType('first-input');
    assert_equals(firstInputs.length, 1, 'There should be a single first-input entry');
    let firstInput = firstInputs[0];
    assert_equals(firstInput.name, entry.name);
    assert_equals(firstInput.entryType, 'first-input');
    assert_equals(firstInput.startTime, entry.startTime);
    assert_equals(firstInput.duration, entry.duration);
    assert_equals(firstInput.processingStart, entry.processingStart);
    assert_equals(firstInput.processingEnd, entry.processingEnd);
    assert_equals(firstInput.cancelable, entry.cancelable);
  }
  if (targetId)
    assert_equals(entry.target, document.getElementById(targetId));
}

function verifyClickEvent(entry, targetId, isFirst=false, minDuration=104, event='pointerdown') {
  verifyEvent(entry, event, targetId, isFirst, minDuration);
}

function wait() {
  return new Promise((resolve, reject) => {
    step_timeout(() => {
      resolve();
    }, 0);
  });
}

function clickAndBlockMain(id) {
  return new Promise((resolve, reject) => {
    clickOnElementAndDelay(id, 120, resolve);
  });
}

function waitForTick() {
  return new Promise(resolve => {
    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(resolve);
    });
  });
}
  // Add a PerformanceObserver and observe with a durationThreshold of |dur|. This test will
  // attempt to check that the duration is appropriately checked by:
  // * Asserting that entries received have a duration which is the smallest multiple of 8
  //   that is greater than or equal to |dur|.
  // * Issuing |numEntries| entries that has duration greater than |slowDur|.
  // * Asserting that exactly |numEntries| entries are received.
  // Parameters:
  // |t|          - the test harness.
  // |dur|        - the durationThreshold for the PerformanceObserver.
  // |id|         - the ID of the element to be clicked.
  // |numEntries| - the number of entries.
  // |slowDur|    - the min duration of a slow entry.
async function testDuration(t, id, numEntries, dur, slowDur) {
  assert_implements(window.PerformanceEventTiming, 'Event Timing is not supported.');
  const observerPromise = new Promise(async resolve => {
    let minDuration = Math.ceil(dur / 8) * 8;
    // Exposed events must always have a minimum duration of 16.
    minDuration = Math.max(minDuration, 16);
    let numEntriesReceived = 0;
    new PerformanceObserver(list => {
      const pointerDowns = list.getEntriesByName('pointerdown');
      pointerDowns.forEach(e => {
        t.step(() => {
          verifyClickEvent(e, id, false /* isFirst */, minDuration);
        });
      });
      numEntriesReceived += pointerDowns.length;
      // All the entries should be received since the slowDur is higher
      // than the duration threshold.
      if (numEntriesReceived === numEntries)
        resolve();
    }).observe({type: "event", durationThreshold: dur});
  });
  const clicksPromise = new Promise(async resolve => {
    for (let index = 0; index < numEntries; index++) {
      // Add some click events that has at least slowDur for duration.
      await clickOnElementAndDelay(id, slowDur);
    }
    resolve();
  });
  return Promise.all([observerPromise, clicksPromise]);
}

  // Add a PerformanceObserver and observe with a durationThreshold of |durThreshold|. This test will
  // attempt to check that the duration is appropriately checked by:
  // * Asserting that entries received have a duration which is the smallest multiple of 8
  //   that is greater than or equal to |durThreshold|.
  // * Issuing |numEntries| entries that have at least |processingDelay| as duration.
  // * Asserting that the entries we receive has duration greater than or equals to the
  //   duration threshold we setup
  // Parameters:
  // |t|                     - the test harness.
  // |id|                    - the ID of the element to be clicked.
  // |durThreshold|          - the durationThreshold for the PerformanceObserver.
  // |numEntries|            - the number of slow and number of fast entries.
  // |processingDelay|       - the event duration we add on each event.
  async function testDurationWithDurationThreshold(t, id, numEntries, durThreshold, processingDelay) {
    assert_implements(window.PerformanceEventTiming, 'Event Timing is not supported.');
    const observerPromise = new Promise(async resolve => {
      let minDuration = Math.ceil(durThreshold / 8) * 8;
      // Exposed events must always have a minimum duration of 16.
      minDuration = Math.max(minDuration, 16);
      new PerformanceObserver(t.step_func(list => {
        const pointerDowns = list.getEntriesByName('pointerdown');
        pointerDowns.forEach(p => {
        assert_greater_than_equal(p.duration, minDuration,
          "The entry's duration should be greater than or equal to " + minDuration + " ms.");
        });
        resolve();
      })).observe({type: "event", durationThreshold: durThreshold});
    });
    for (let index = 0; index < numEntries; index++) {
      // These clicks are expected to be ignored, unless the test has some extra delays.
      // In that case, the test will verify the event duration to ensure the event duration is
      // greater than the duration threshold
      await clickOnElementAndDelay(id, processingDelay);
    }
    // Send click with event duration equals to or greater than |durThreshold|, so the
    // observer promise can be resolved
    await clickOnElementAndDelay(id, durThreshold);
    return observerPromise;
  }

// Apply events that trigger an event of the given |eventType| to be dispatched to the
// |target|. Some of these assume that the target is not on the top left corner of the
// screen, which means that (0, 0) of the viewport is outside of the |target|.
function applyAction(eventType, target) {
  const actions = new test_driver.Actions();
  if (eventType === 'auxclick') {
    actions.pointerMove(0, 0, {origin: target})
    .pointerDown({button: actions.ButtonType.MIDDLE})
    .pointerUp({button: actions.ButtonType.MIDDLE});
  } else if (eventType === 'click' || eventType === 'mousedown' || eventType === 'mouseup'
      || eventType === 'pointerdown' || eventType === 'pointerup'
      || eventType === 'touchstart' || eventType === 'touchend') {
    actions.pointerMove(0, 0, {origin: target})
    .pointerDown()
    .pointerUp();
  } else if (eventType === 'contextmenu') {
    actions.pointerMove(0, 0, {origin: target})
    .pointerDown({button: actions.ButtonType.RIGHT})
    .pointerUp({button: actions.ButtonType.RIGHT});
  } else if (eventType === 'dblclick') {
    actions.pointerMove(0, 0, {origin: target})
    .pointerDown()
    .pointerUp()
    .pointerDown()
    .pointerUp()
    // Reset by clicking outside of the target.
    .pointerMove(0, 0)
    .pointerDown()
  } else if (eventType === 'mouseenter' || eventType === 'mouseover'
      || eventType === 'pointerenter' || eventType === 'pointerover') {
    // Move outside of the target and then back inside.
    // Moving it to 0, 1 because 0, 0 doesn't cause the pointer to
    // move in Firefox. See https://github.com/w3c/webdriver/issues/1545
    actions.pointerMove(0, 1)
    .pointerMove(0, 0, {origin: target});
  } else if (eventType === 'mouseleave' || eventType === 'mouseout'
      || eventType === 'pointerleave' || eventType === 'pointerout') {
    actions.pointerMove(0, 0, {origin: target})
    .pointerMove(0, 0);
  } else if (eventType === 'keyup' || eventType === 'keydown') {
    // Any key here as an input should work.
    // TODO: Switch this to use test_driver.Actions.key{up,down}
    // when test driver supports it.
    // Please check crbug.com/893480.
    const key = 'k';
    return test_driver.send_keys(target, key);
  } else {
    assert_unreached('The event type ' + eventType + ' is not supported.');
  }
  return actions.send();
}

function requiresListener(eventType) {
  return ['mouseenter',
          'mouseleave',
          'pointerdown',
          'pointerenter',
          'pointerleave',
          'pointerout',
          'pointerover',
          'pointerup',
          'keyup',
          'keydown'
        ].includes(eventType);
}

function notCancelable(eventType) {
  return ['mouseenter', 'mouseleave', 'pointerenter', 'pointerleave'].includes(eventType);
}

// Tests the given |eventType|'s performance.eventCounts value. Since this is populated only when
// the event is processed, we check every 10 ms until we've found the |expectedCount|.
function testCounts(t, resolve, looseCount, eventType, expectedCount) {
  const counts = performance.eventCounts.get(eventType);
  if (counts < expectedCount) {
    t.step_timeout(() => {
      testCounts(t, resolve, looseCount, eventType, expectedCount);
    }, 10);
    return;
  }
  if (looseCount) {
    assert_greater_than_equal(performance.eventCounts.get(eventType), expectedCount,
        `Should have at least ${expectedCount} ${eventType} events`)
  } else {
    assert_equals(performance.eventCounts.get(eventType), expectedCount,
        `Should have ${expectedCount} ${eventType} events`);
  }
  resolve();
}

// Tests the given |eventType| by creating events whose target are the element with id
// 'target'. The test assumes that such element already exists. |looseCount| is set for
// eventTypes for which events would occur for other interactions other than the ones being
// specified for the target, so the counts could be larger.
async function testEventType(t, eventType, looseCount=false) {
  assert_implements(window.EventCounts, "Event Counts isn't supported");
  const target = document.getElementById('target');
  if (requiresListener(eventType)) {
    target.addEventListener(eventType, () =>{});
  }
  const initialCount = performance.eventCounts.get(eventType);
  if (!looseCount) {
    assert_equals(initialCount, 0, 'No events yet.');
  }
  // Trigger two 'fast' events of the type.
  await applyAction(eventType, target);
  await applyAction(eventType, target);
  await waitForTick();
  await new Promise(t.step_func(resolve => {
    testCounts(t, resolve, looseCount, eventType, initialCount + 2);
  }));
  // The durationThreshold used by the observer. A slow events needs to be slower than that.
  const durationThreshold = 16;
  // Now add an event handler to cause a slow event.
  target.addEventListener(eventType, () => {
    mainThreadBusy(durationThreshold + 4);
  });
  const observerPromise = new Promise(async resolve => {
    new PerformanceObserver(t.step_func(entryList => {
      let eventTypeEntries = entryList.getEntriesByName(eventType);
      if (eventTypeEntries.length === 0)
        return;

      let entry = null;
      if (!looseCount) {
        entry = eventTypeEntries[0];
        assert_equals(eventTypeEntries.length, 1);
      } else {
        // The other events could also be considered slow. Find the one with the correct
        // target.
        eventTypeEntries.forEach(e => {
          if (e.target === document.getElementById('target'))
            entry = e;
        });
        if (!entry)
          return;
      }
      verifyEvent(entry,
                  eventType,
                  'target',
                  false /* isFirst */,
                  durationThreshold,
                  notCancelable(eventType));
      // Shouldn't need async testing here since we already got the observer entry, but might as
      // well reuse the method.
      testCounts(t, resolve, looseCount, eventType, initialCount + 3);
    })).observe({type: 'event', durationThreshold: durationThreshold});
  });
  // Cause a slow event.
  await applyAction(eventType, target);

  await waitForTick();

  await observerPromise;
}

function addListeners(element, events) {
  const clickHandler = (e) => {
    mainThreadBusy(200);
  };
  events.forEach(e => { element.addEventListener(e, clickHandler); });
}

// The testdriver.js, testdriver-vendor.js and testdriver-actions.js need to be
// included to use this function.
async function tap(element) {
  return new test_driver.Actions()
    .addPointer("touchPointer", "touch")
    .pointerMove(0, 0, { origin: element })
    .pointerDown()
    .pointerUp()
    .send();
}

// The testdriver.js, testdriver-vendor.js need to be included to use this
// function.
async function pressKey(element, key) {
  await test_driver.send_keys(element, key);
}

// The testdriver.js, testdriver-vendor.js need to be included to use this
// function.
async function addListenersAndPress(element, key, events) {
  addListeners(element, events);
  return pressKey(element, key);
}

// The testdriver.js, testdriver-vendor.js need to be included to use this
// function.
async function addListenersAndClick(element) {
  addListeners(element,
    ['mousedown', 'mouseup', 'pointerdown', 'pointerup', 'click']);
  return test_driver.click(element);
}

function filterAndAddToMap(events, map) {
  return function (entry) {
    if (events.includes(entry.name)) {
      map.set(entry.name, entry.interactionId);
      return true;
    }
    return false;
  }
}

async function createPerformanceObserverPromise(observeTypes, callback, readyToResolve
) {
  return new Promise(resolve => {
    new PerformanceObserver(entryList => {
      callback(entryList);

      if (readyToResolve()) {
        resolve();
      }
    }).observe({ entryTypes: observeTypes });
  });
}

// The testdriver.js, testdriver-vendor.js need to be included to use this
// function.
async function interactAndObserve(interactionType, element, observerPromise) {
  let interactionPromise;
  switch (interactionType) {
    case 'tap': {
      addListeners(element, ['pointerdown', 'pointerup']);
      interactionPromise = tap(element);
      break;
    }
    case 'click': {
      addListeners(element,
        ['mousedown', 'mouseup', 'pointerdown', 'pointerup', 'click']);
      interactionPromise = test_driver.click(element);
      break;
    }
  }
  return Promise.all([interactionPromise, observerPromise]);
}

async function interact(interactionType, element, key = '') {
  switch (interactionType) {
    case 'click': {
      return test_driver.click(element);
    }
    case 'tap': {
      return tap(element);
    }
    case 'key': {
      return test_driver.send_keys(element, key);
    }
  }
}

async function verifyInteractionCount(t, expectedCount) {
  await t.step_wait(() => {
    return performance.interactionCount >= expectedCount;
  }, 'interactionCount did not increase enough', 10000, 5);
  assert_equals(performance.interactionCount, expectedCount,
    'interactionCount increased more than expected');
}

function interactionCount_test(interactionType, elements, key = '') {
  return promise_test(async t => {
    assert_implements(window.PerformanceEventTiming,
      'Event Timing is not supported');
    assert_equals(performance.interactionCount, 0, 'Initial count is not 0');

    let expectedCount = 1;
    for (let element of elements) {
      await interact(interactionType, element, key);
      await verifyInteractionCount(t, expectedCount++);
    }
  }, `EventTiming: verify interactionCount for ${interactionType} interaction`);
}
