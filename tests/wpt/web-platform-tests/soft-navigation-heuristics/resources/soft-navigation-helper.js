var counter = 0;
var clicked;
var timestamps = []
const MAX_CLICKS = 50;
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
      promise_test(async t => {
        const preClickLcp = await getLcpEntries();
        setEvent(t, link, pushState, addContent, pushUrl, eventType);
        for (let i = 0; i < clicks; ++i) {
          clicked = false;
          click(link);

          await new Promise(resolve => {
            (new PerformanceObserver(() => resolve())).observe({
              type: 'soft-navigation'
            });
          });
        }
        assert_equals(
            document.softNavigations, clicks,
            'Soft Navigations detected are the same as the number of clicks');
        await validateSoftNavigationEntry(
            clicks, extraValidations, pushUrl);

        await runEntryValidations(preClickLcp);
      }, testName);
    };

const testNavigationApi = (testName, navigateEventHandler, link) => {
  promise_test(async t => {
    const preClickLcp = await getLcpEntries();
    navigation.addEventListener('navigate', navigateEventHandler);
    click(link);
    await new Promise(resolve => {
      (new PerformanceObserver(() => resolve())).observe({
        type: 'soft-navigation'
      });
    });
    assert_equals(document.softNavigations, 1, 'Soft Navigation detected');
    await validateSoftNavigationEntry(1, () => {}, 'foobar.html');

    await runEntryValidations(preClickLcp);
  }, testName);
};

const testSoftNavigationNotDetected = options => {
    promise_test(async t => {
      const preClickLcp = await getLcpEntries();
      options.eventTarget.addEventListener(options.eventName, options.eventHandler);
      click(options.link);
      await new Promise((resolve, reject) => {
        (new PerformanceObserver(() =>
            reject("Soft navigation should not be triggered"))).observe({
          type: 'soft-navigation',
          buffered: true
        });
        t.step_timeout(resolve, 1000);
      });
      assert_equals(
          document.softNavigations, 0, 'Soft Navigation not detected');
    }, options.testName);
  };

const runEntryValidations = async preClickLcp => {
  await doubleRaf();
  validatePaintEntries('first-contentful-paint');
  validatePaintEntries('first-paint');
  const postClickLcp = await getLcpEntries();
  assert_greater_than(
      postClickLcp.length, preClickLcp.length,
      'Soft navigation should have triggered at least an LCP entry');
  assert_not_equals(
      postClickLcp[postClickLcp.length - 1].size,
      preClickLcp[preClickLcp.length - 1].size,
      'Soft navigation LCP element should not have identical size to the hard ' +
          'navigation LCP element');
};

const click = link => {
  if (test_driver) {
    test_driver.click(link);
    timestamps[counter] = {"syncPostClick": performance.now()};
  }
}

const doubleRaf = () => {
  return new Promise(r => {
    requestAnimationFrame(()=>requestAnimationFrame(r));
  });
};

const setEvent = (t, button, pushState, addContent, pushUrl, eventType) => {
  const eventObject = (eventType == "click") ? button : window;
  eventObject.addEventListener(eventType, async e => {
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

    clicked = true;
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
    assert_less_than_equal(timestamps[i]["syncPostClick"], entryTimestamp);
    assert_greater_than_equal(
        timestamps[i]['eventStart'], entryTimestamp,
        'Event start timestamp matches');
    assert_not_equals(entry.navigationId,
                      performance.getEntriesByType("navigation")[0].navigationId,
                      "The navigation ID was incremented");
    if (i > 0) {
      assert_not_equals(entry.navigationId,
                        entries[i-1].navigationId,
                        "The navigation ID was incremented between clicks");
    }
  }
  assert_equals(performance.getEntriesByType("soft-navigation").length,
                expectedClicks, "Performance timeline got an entry");
  extraValidations(entries, options);

};

const validatePaintEntries = async type => {
  const entries = await new Promise(resolve => {
    (new PerformanceObserver(list => resolve(
      list.getEntriesByName(type)))).observe(
      {type: 'paint', buffered: true});
    });
  // TODO(crbug/1372997): investigate why this is not failing when multiple
  // clicks are fired. Also, make sure the observer waits on the number of
  // required clicks, instead of counting on double rAF.
  assert_equals(entries.length, 2, "There are two entries for " + type);
  assert_not_equals(entries[0].startTime, entries[1].startTime,
    "Entries have different timestamps for " + type);
};

const getLcpEntries = async () => {
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
  await img.decode();
  element.appendChild(img);
};
const addImageToMain = async () => {
  await addImage(document.getElementById('main'));
};

const addTextToDivOnMain = () => {
  const main = document.getElementById("main");
  const prevDiv = document.getElementsByTagName("div")[0];
  if (prevDiv) {
    main.removeChild(prevDiv);
  }
  const div = document.createElement("div");
  const text = document.createTextNode("Lorem Ipsum");
  div.appendChild(text);
  div.style="font-size: 3em";
  main.appendChild(div);
}
