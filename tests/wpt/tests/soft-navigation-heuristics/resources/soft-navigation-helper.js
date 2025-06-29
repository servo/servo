var counter = 0;
var timestamps = [];

const SOFT_NAV_ENTRY_BUFFER_LIMIT = 50;
// this is used by injected scripts
const DEFAULTURL = 'foobar.html';
const DEFAULTIMG = '/soft-navigation-heuristics/resources/images/lcp-256x256-alt-1.png';

/**
 * Common Utils not related to these tests.
 * TODO: Could be moved out?
 */

// Helper method for use with history.back(), when we want to be
// sure that its asynchronous effect has completed.
async function waitForUrlToEndWith(url) {
  return new Promise((resolve, reject) => {
    window.addEventListener('popstate', () => {
      if (location.href.endsWith(url)) {
        resolve();
      } else {
        reject(
          'Got ' + location.href + ' - expected URL ends with "' + url + '"');
      }
    }, { once: true });
  });
};

function getNextEntry(type) {
  return new Promise(resolve => {
    new PerformanceObserver((list, observer) => {
      const entries = list.getEntries();
      observer.disconnect();
      assert_equals(entries.length, 1, 'Only one entry.');
      resolve(entries[0]);
    }).observe({ type, includeSoftNavigationObservations: true });
  });
}

function getBufferedEntries(type) {
  return new Promise((resolve, reject) => {
    new PerformanceObserver((list, observer, options) => {
      if (options.droppedEntriesCount) {
        reject(options.droppedEntriesCount);
      }
      resolve(list.getEntries());
      observer.disconnect();
    }).observe({ type, buffered: true, includeSoftNavigationObservations: true });
  });
}

/**
 * Helpers somewhat specific to these test types, "exported" and used by tests.
 */

async function addImageToMain(url = DEFAULTIMG, id = 'imagelcp') {
  const main = document.getElementById('main');
  const img = new Image();
  img.src = url + '?' + Math.random();
  img.id = id;
  img.setAttribute('elementtiming', id);
  main.appendChild(img);
  return img;
}

function addTextParagraphToMain(text, element_timing = '') {
  const main = document.getElementById('main');
  const p = document.createElement('p');
  const textNode = document.createTextNode(text);
  p.setAttribute('elementtiming', element_timing);
  p.style = 'font-size: 3em';
  p.appendChild(textNode);
  main.appendChild(p);
  return p;
}

function addTextToDivOnMain() {
  const main = document.getElementById('main');
  const prevDiv = document.getElementsByTagName('div')[0];
  if (prevDiv) {
    main.removeChild(prevDiv);
  }
  const div = document.createElement('div');
  const text = document.createTextNode('Lorem Ipsum');
  div.style = 'font-size: 3em';
  div.appendChild(text);
  main.appendChild(div);
  return div;
}


/**
 * Internal Helpers
 */

async function _withTimeoutMessage(t, promise, message, timeout = 1000) {
  return Promise.race([
    promise,
    new Promise((resolve, reject) => {
      t.step_timeout(() => {
        reject(new Error(message));
      }, timeout);
    }),
  ]);
}

function _maybeAddUrlCleanupForTesting(t, numClicks) {
  // TODO: any way to early-exit if we are running headless?
  if (numClicks > 50) return;
  t.add_cleanup(async () => {
    // Go back to the original URL
    for (let i = 0; i < numClicks; i++) {
      history.back();
      await new Promise(resolve => {
        addEventListener('popstate', resolve, { once: true });
      });
    }
  });
}


/**
 * Test body and validations
 */

function testSoftNavigation(options) {
  const testName = options.testName;
  if (!testName) throw new Error("testName is a required option.");

  promise_test(async t => {
    const {
      clickTarget = document.getElementById("link"),
      eventListenerCb = () => { },
      interactionFunc = () => { if (test_driver) test_driver.click(clickTarget); },
      registerInteractionEvent = (cb) => clickTarget.addEventListener('click', cb),
      registerRouteChange = (cb) => registerInteractionEvent(async (event) => {
        // The default route change handler is ClickEvent + Yield, in order to:
        // - mark timeOrigin.
        // - ensure task tracking is working properly.
        await new Promise(r => t.step_timeout(r, 0));
        cb(event);
      }),
      numClicks = 1,

      addContent = () => addTextParagraphToMain(),
      clearContent = () => { },
      pushState = url => { history.pushState({}, '', url); },
      pushUrl = DEFAULTURL,
      dontExpectSoftNavs = false,
      onRouteChange = async (event) => {
        await pushState(`${pushUrl}?${counter}`);
        // Wait 10 ms to make sure the timestamps are correct.
        await new Promise(r => t.step_timeout(r, 10));
        await clearContent();
        await addContent();
      },

      extraSetup = () => { },
      extraValidations = () => { },
    } = options;

    _maybeAddUrlCleanupForTesting(t);

    await extraSetup(t);

    // Allow things to settle before starting the test.  Specifically,
    // wait for final LCP candidate to arrive.
    // TODO: Make this explicitly wait by marking the candidate, or just making
    // the image `blocking=rendering`?
    await new Promise((r) => {
      requestAnimationFrame(() => {
        t.step_timeout(r, 1000);
      })
    })

    const lcps_before = await _withTimeoutMessage(t,
      getBufferedEntries('largest-contentful-paint'),
      'Timed out waiting for LCP entries');

    // This "click event" starts the user interaction.
    registerInteractionEvent(async event => {
      eventListenerCb(event);

      // Event listener is no-op and yields immediately. Mark its sync end time:
      // TODO: This is very brittle, as some tests "customize" it.
      if (!timestamps[counter]['eventEnd']) {
        timestamps[counter]['eventEnd'] = performance.now();
      }
    });

    // This "route event" starts the UI/URL changes.  Often also the event.
    registerRouteChange(async event => {
      await onRouteChange(event);
      ++counter;
    });

    const softNavEntries = [];
    const icps = [];
    for (let i = 0; i < numClicks; ++i) {
      // Use getNextEntry instead of getBufferedEntries so that:
      // - For tests with more than 1 click, we wait for all expectations
      //   to arrive between clicks
      // - For tests with more than buffer-limit clicks, we actually measure.
      const soft_nav_promise = getNextEntry('soft-navigation');
      const icp_promise = getNextEntry('interaction-contentful-paint');

      await interactionFunc();
      timestamps[counter] = { 'syncPostInteraction': performance.now() };

      // TODO: is it possible to still await these entries, but change to
      // expect a timeout without resolution, to actually expect non arrives?
      if (dontExpectSoftNavs) continue;

      softNavEntries.push(await _withTimeoutMessage(t,
        soft_nav_promise, 'Timed out waiting for soft navigation', 3000));

      icps.push(await _withTimeoutMessage(t,
        icp_promise, 'Timed out waiting for icp', 3000));
    }

    const lcps_after = await getBufferedEntries('largest-contentful-paint');

    const expectedNumberOfSoftNavs = (dontExpectSoftNavs) ? 0 : numClicks;

    await _withTimeoutMessage(t,
      validateSoftNavigationEntries(t, softNavEntries, expectedNumberOfSoftNavs, pushUrl),
      'Timed out waiting for soft navigation entry validation');

    await _withTimeoutMessage(t,
      validateIcpEntries(t, softNavEntries, lcps_before, icps, lcps_after),
      'Timed out waiting for ICP entry validations');

    await _withTimeoutMessage(t,
      extraValidations(t, softNavEntries, lcps_before, icps),
      'Timed out waiting for extra validations');
  }, testName);
}

// TODO: Find a way to remove the need for this
function testNavigationApi(testName, navigateEventHandler, link) {
  navigation.addEventListener('navigate', navigateEventHandler);
  testSoftNavigation({
    testName,
    link,
    pushState: () => { },
  });
}

async function validateSoftNavigationEntries(t, softNavEntries, expectedNumSoftNavs, pushUrl) {
  assert_equals(softNavEntries.length, expectedNumSoftNavs,
    'Soft Navigations detected are the same as the number of clicks');

  const hardNavEntry = performance.getEntriesByType('navigation')[0];
  const all_navigation_ids = new Set(
    [hardNavEntry.navigationId, ...softNavEntries.map(entry => entry.navigationId)]);

  assert_equals(
    all_navigation_ids.size, expectedNumSoftNavs + 1,
    'The navigation ID was re-generated between all hard and soft navs');

  if (expectedNumSoftNavs > SOFT_NAV_ENTRY_BUFFER_LIMIT) {
    // TODO: Consider exposing args to `extraValidationsSN` so the
    // dropped entry count test can make these assertions directly.
    // Having it here has the advantage of testing ALL tests, but, it has
    // the disadvantage of not being able to assert that for sure we hit this
    // code path in that specific test.  (tested locally that it does, but
    // what if buffer sizes change in the future?)
    const expectedDroppedEntriesCount = expectedNumSoftNavs - SOFT_NAV_ENTRY_BUFFER_LIMIT;
    await promise_rejects_exactly(t, expectedDroppedEntriesCount,
      getBufferedEntries('soft-navigation'),
      "This should reject with the number of dropped entries")
  }

  for (let i = 0; i < softNavEntries.length; ++i) {
    const softNavEntry = softNavEntries[i];
    assert_regexp_match(
      softNavEntry.name, new RegExp(pushUrl),
      'The soft navigation name is properly set');

    // TODO: Carefully look at these and re-enable, also: assert_between_inclusive
    // const timeOrigin = softNavEntry.startTime;
    // assert_greater_than_equal(
    //   timeOrigin, timestamps[i]['eventEnd'],
    //   'Event start timestamp matches');
    // assert_less_than_equal(
    //   timeOrigin, timestamps[i]['syncPostInteraction'],
    //   'Entry timestamp is lower than the post interaction one');
  }
}


async function validateIcpEntries(t, softNavEntries, lcps, icps, lcps_after) {
  assert_equals(
    lcps.length, lcps_after.length,
    'Soft navigation should not have triggered more LCP entries.');

  assert_greater_than_equal(
    icps.length, softNavEntries.length,
    'Should have at least one ICP entry per soft navigation.');

  const lcp = lcps.at(-1);

  // Group ICP entries by their navigation ID.
  const icpsByNavId = new Map();
  for (const icp of icps) {
    if (!icpsByNavId.has(icp.navigationId)) {
      icpsByNavId.set(icp.navigationId, []);
    }
    icpsByNavId.get(icp.navigationId).push(icp);
  }

  // For each soft navigation, find and validate its corresponding ICP entry.
  for (const softNav of softNavEntries) {
    const navId = softNav.navigationId;
    assert_true(icpsByNavId.has(navId),
      `An ICP entry should be present for navigationId ${navId}`);

    // Get the largest ICP entry for this specific navigation.
    // TODO: validate multiple candidates (i.e. each is newer + larger).
    const icp = icpsByNavId.get(navId).at(-1);

    assert_not_equals(lcp.size, icp.size,
      `LCP element should not have identical size to ICP element for navigationId ${navId}.`);
    assert_not_equals(lcp.startTime, icp.startTime,
      `LCP element should not have identical startTime to ICP element for navigationId ${navId}.`);
  }
}


// Receives an image InteractionContentfulPaint |entry| and checks |entry|'s attribute values.
// The |timeLowerBound| parameter is a lower bound on the loadTime value of the entry.
// The |options| parameter may contain some string values specifying the following:
// * 'renderTimeIs0': the renderTime should be 0 (image does not pass Timing-Allow-Origin checks).
//     When not present, the renderTime should not be 0 (image passes the checks).
// * 'sizeLowerBound': the |expectedSize| is only a lower bound on the size attribute value.
//     When not present, |expectedSize| must be exactly equal to the size attribute value.
// * 'approximateSize': the |expectedSize| is only approximate to the size attribute value.
//     This option is mutually exclusive to 'sizeLowerBound'.
function checkImage(entry, expectedUrl, expectedID, expectedSize, timeLowerBound, options = []) {
  assert_equals(entry.name, '', "Entry name should be the empty string");
  assert_equals(entry.entryType, 'interaction-contentful-paint',
    "Entry type should be interaction-contentful-paint");
  assert_equals(entry.duration, 0, "Entry duration should be 0");
  // The entry's url can be truncated.
  assert_equals(expectedUrl.substr(0, 100), entry.url.substr(0, 100),
    `Expected URL ${expectedUrl} should at least start with the entry's URL ${entry.url}`);
  assert_equals(entry.id, expectedID, "Entry ID matches expected one");
  assert_equals(entry.element, document.getElementById(expectedID),
    "Entry element is expected one");
  if (options.includes('skip')) {
    return;
  }
  assert_greater_than_equal(performance.now(), entry.renderTime,
    'renderTime should occur before the entry is dispatched to the observer.');
  assert_approx_equals(entry.startTime, entry.renderTime, 0.001,
    'startTime should be equal to renderTime to the precision of 1 millisecond.');
  if (options.includes('sizeLowerBound')) {
    assert_greater_than(entry.size, expectedSize);
  } else if (options.includes('approximateSize')) {
    assert_approx_equals(entry.size, expectedSize, 1);
  } else {
    assert_equals(entry.size, expectedSize);
  }

  assert_greater_than_equal(entry.paintTime, timeLowerBound,
    'paintTime should represent the time when the UA started painting');

  // PaintTimingMixin
  if ("presentationTime" in entry && entry.presentationTime !== null) {
    assert_greater_than(entry.presentationTime, entry.paintTime);
    assert_equals(entry.presentationTime, entry.renderTime);
  } else {
    assert_equals(entry.renderTime, entry.paintTime);
  }

  if (options.includes('animated')) {
    assert_less_than(entry.renderTime, image_delay,
      'renderTime should be smaller than the delay applied to the second frame');
    assert_greater_than(entry.renderTime, 0,
      'renderTime should be larger than 0');
  }
  else {
    assert_between_inclusive(entry.loadTime, timeLowerBound, entry.renderTime,
      'loadTime should occur between the lower bound and the renderTime');
  }
}
