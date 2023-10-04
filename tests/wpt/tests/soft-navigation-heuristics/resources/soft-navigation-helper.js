var counter = 0;
var interacted;
var timestamps = []
const MAX_CLICKS = 50;
// Entries for one hard navigation + 50 soft navigations.
const MAX_PAINT_ENTRIES = 51;
const URL = "foobar.html";
const readValue = (value, defaultValue) => {
  return value != undefined ? value : defaultValue;
}
const testSoftNavigation =
    options => {
      const addContent = options.addContent;
      const link = options.link;
      const pushState = readValue(options.pushState,
        url=>{history.pushState({}, '', url)});
      const clicks = readValue(options.clicks, 1);
      const extraValidations = readValue(options.extraValidations,
                                                   () => {});
      const testName = options.testName;
      const pushUrl = readValue(options.pushUrl, true);
      const eventType = readValue(options.eventType, "click");
      const interactionType = readValue(options.interactionType, 'click');
      const expectLCP = options.validate != 'no-lcp';
      const eventPrepWork = options.eventPrepWork;
      promise_test(async t => {
        await waitInitialLCP();
        const preClickLcp = await getLcpEntries();
        setEvent(t, link, pushState, addContent, pushUrl, eventType, eventPrepWork);
        for (let i = 0; i < clicks; ++i) {
          const firstClick = (i === 0);
          let paint_entries_promise =
              waitOnPaintEntriesPromise(expectLCP && firstClick);
          interacted = false;
          interact(link, interactionType);

          await new Promise(resolve => {
            (new PerformanceObserver(() => resolve())).observe({
              type: 'soft-navigation'
            });
          });
          // Ensure paint timing entries are fired before moving on to the next
          // click.
          await paint_entries_promise;
        }
        assert_equals(
            document.softNavigations, clicks,
            'Soft Navigations detected are the same as the number of clicks');
        await validateSoftNavigationEntry(
            clicks, extraValidations, pushUrl);

        await runEntryValidations(preClickLcp, clicks + 1, expectLCP);
      }, testName);
    };

const testNavigationApi = (testName, navigateEventHandler, link) => {
  promise_test(async t => {
    navigation.addEventListener('navigate', navigateEventHandler);
    const navigated = new Promise(resolve => {
      navigation.addEventListener('navigatesuccess', resolve);
      navigation.addEventListener('navigateerror', resolve);
    });
    await waitInitialLCP();
    const preClickLcp = await getLcpEntries();
    let paint_entries_promise = waitOnPaintEntriesPromise();
    interact(link);
    await new Promise(resolve => {
      (new PerformanceObserver(() => resolve())).observe({
        type: 'soft-navigation'
      });
    });
    await navigated;
    await paint_entries_promise;
    assert_equals(document.softNavigations, 1, 'Soft Navigation detected');
    await validateSoftNavigationEntry(1, () => {}, 'foobar.html');

    await runEntryValidations(preClickLcp);
  }, testName);
};

const testSoftNavigationNotDetected = options => {
    promise_test(async t => {
      const preClickLcp = await getLcpEntries();
      options.eventTarget.addEventListener(options.eventName, options.eventHandler);
      interact(options.link);
      await new Promise((resolve, reject) => {
        (new PerformanceObserver(() =>
            reject("Soft navigation should not be triggered"))).observe({
          type: 'soft-navigation',
          buffered: true
        });
        t.step_timeout(resolve, 1000);
      });
      if (document.softNavigations) {
        assert_equals(
          document.softNavigations, 0, 'Soft Navigation not detected');
      }
      const postClickLcp = await getLcpEntries();
      assert_equals(
          preClickLcp.length, postClickLcp.length, 'No LCP entries accumulated');
    }, options.testName);
  };

const runEntryValidations =
    async (preClickLcp, entries_expected_number = 2, expect_lcp = true) => {
  await validatePaintEntries('first-contentful-paint', entries_expected_number);
  await validatePaintEntries('first-paint', entries_expected_number);
  const postClickLcp = await getLcpEntries();
  const postClickLcpWithoutSoftNavs = await getLcpEntriesWithoutSoftNavs();
  if (expect_lcp) {
    assert_greater_than(
        postClickLcp.length, preClickLcp.length,
        'Soft navigation should have triggered at least an LCP entry');
  } else {
    assert_equals(
        postClickLcp.length, preClickLcp.length,
        'Soft navigation should not have triggered an LCP entry');
  }
  assert_equals(
      postClickLcpWithoutSoftNavs.length, preClickLcp.length,
      'Soft navigation should not have triggered an LCP entry when the ' +
      'observer did not opt in');
  if (expect_lcp) {
    assert_not_equals(
        postClickLcp[postClickLcp.length - 1].size,
        preClickLcp[preClickLcp.length - 1].size,
        'Soft navigation LCP element should not have identical size to the hard ' +
            'navigation LCP element');
  } else {
    assert_equals(
        postClickLcp[postClickLcp.length - 1].size,
        preClickLcp[preClickLcp.length - 1].size,
        'Soft navigation LCP element should have an identical size to the hard ' +
            'navigation LCP element');
  }
};

const interact =
    (link, interactionType = 'click') => {
      if (test_driver) {
        if (interactionType == 'click') {
          test_driver.click(link);
        } else {
          test_driver.send_keys(link, 'j');
        }
        timestamps[counter] = {"syncPostInteraction": performance.now()};
      }
    }

const setEvent = (t, button, pushState, addContent, pushUrl, eventType, prepWork) => {
  const eventObject =
      (eventType == 'click' || eventType == 'keydown') ? button : window;
  eventObject.addEventListener(eventType, async e => {
    if (prepWork &&!prepWork(t)) {
      return;
    }
    timestamps[counter]["eventStart"] = performance.now();
    // Jump through a task, to ensure task tracking is working properly.
    await new Promise(r => t.step_timeout(r, 0));

    const url = URL + "?" + counter;
    if (pushState) {
      // Change the URL
      if (pushUrl) {
        pushState(url);
      } else {
        pushState();
      }
    }

    // Wait 10 ms to make sure the timestamps are correct.
    await new Promise(r => t.step_timeout(r, 10));

    await addContent(url);
    ++counter;

    interacted = true;
  });
};

const validateSoftNavigationEntry = async (clicks, extraValidations,
                                              pushUrl) => {
  const [entries, options] = await new Promise(resolve => {
    (new PerformanceObserver((list, obs, options) => resolve(
      [list.getEntries(), options]))).observe(
      {type: 'soft-navigation', buffered: true});
    });
  const expectedClicks = Math.min(clicks, MAX_CLICKS);

  assert_equals(entries.length, expectedClicks,
                "Performance observer got an entry");
  for (let i = 0; i < entries.length; ++i) {
    const entry = entries[i];
    assert_true(entry.name.includes(pushUrl ? URL : document.location.href),
                "The soft navigation name is properly set");
    const entryTimestamp = entry.startTime;
    assert_less_than_equal(timestamps[i]["syncPostInteraction"], entryTimestamp);
    assert_greater_than_equal(
        timestamps[i]['eventStart'], entryTimestamp,
        'Event start timestamp matches');
    assert_not_equals(entry.navigationId,
                      performance.getEntriesByType("navigation")[0].navigationId,
      "The navigation ID was re-generated and different from the initial one.");
    if (i > 0) {
      assert_not_equals(entry.navigationId,
                        entries[i-1].navigationId,
        "The navigation ID was re-generated between clicks");
    }
  }
  assert_equals(performance.getEntriesByType("soft-navigation").length,
                expectedClicks, "Performance timeline got an entry");
  await extraValidations(entries, options);

};

const validatePaintEntries = async (type, entries_number) => {
  if (!performance.softNavPaintMetricsSupported) {
    return;
  }
  const expected_entries_number = Math.min(entries_number, MAX_PAINT_ENTRIES);
  const entries = await new Promise(resolve => {
    const entries = [];
    (new PerformanceObserver(list => {
      entries.push(...list.getEntriesByName(type));
      if (entries.length >= expected_entries_number) {
        resolve(entries);
      }
    })).observe(
      {type: 'paint', buffered: true, includeSoftNavigationObservations: true});
    });
  const entries_without_softnavs = await new Promise(resolve => {
    (new PerformanceObserver(list => resolve(
      list.getEntriesByName(type)))).observe(
      {type: 'paint', buffered: true});
    });
  assert_equals(entries.length, expected_entries_number,
    `There are ${entries_number} entries for ${type}`);
  assert_equals(entries_without_softnavs.length, 1,
    `There is one non-softnav entry for ${type}`);
  if (entries_number > 1) {
    assert_not_equals(entries[0].startTime, entries[1].startTime,
      "Entries have different timestamps for " + type);
  }
};

const waitInitialLCP = () => {
  return new Promise(resolve => {
      new PerformanceObserver(list => resolve()).observe({
        type: 'largest-contentful-paint',
        buffered: true
      });
  });
}

const getLcpEntries = async () => {
  const entries = await new Promise(resolve => {
    (new PerformanceObserver(list => resolve(
      list.getEntries()))).observe(
      {type: 'largest-contentful-paint', buffered: true,
       includeSoftNavigationObservations: true});
    });
  return entries;
};

const getLcpEntriesWithoutSoftNavs = async () => {
  const entries = await new Promise(resolve => {
    (new PerformanceObserver(list => resolve(
      list.getEntries()))).observe(
      {type: 'largest-contentful-paint', buffered: true});
    });
  return entries;
};

const addImage = async (element) => {
  const img = new Image();
  img.src = '/images/blue.png' + "?" + Math.random();
  img.id="imagelcp";
  await img.decode();
  element.appendChild(img);
};
const addImageToMain = async () => {
  await addImage(document.getElementById('main'));
};

const addTextToDivOnMain =
    () => {
      const main = document.getElementById("main");
      const prevDiv = document.getElementsByTagName("div")[0];
      if (prevDiv) {
        main.removeChild(prevDiv);
      }
      const div = document.createElement("div");
      const text = document.createTextNode("Lorem Ipsum");
      div.appendChild(text);
      div.style = "font-size: 3em";
      main.appendChild(div);
    }

const waitOnPaintEntriesPromise = (expectLCP = true) => {
  return new Promise((resolve, reject) => {
    if (performance.softNavPaintMetricsSupported) {
      const paint_entries = []
      new PerformanceObserver(list => {
        paint_entries.push(...list.getEntries());
        if (paint_entries.length == 2) {
          resolve();
        } else if (paint_entries.length > 2) {
          reject();
        }
      }).observe({type: 'paint', includeSoftNavigationObservations: true});
    } else if (expectLCP) {
        new PerformanceObserver(list => {
          resolve();
        }).observe({
          type: 'largest-contentful-paint',
          includeSoftNavigationObservations: true
        });
    } else {
        step_timeout(
            () => requestAnimationFrame(() => requestAnimationFrame(resolve)),
            100);
    }
  });
};
