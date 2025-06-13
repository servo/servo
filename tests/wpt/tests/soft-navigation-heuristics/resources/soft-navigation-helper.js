var counter = 0;
var interacted;
var timestamps = [];
const MAX_CLICKS = 50;
const URL = 'foobar.html';
const readValue = (value, defaultValue) => {
  return value !== undefined ? value : defaultValue;
};

const withTimeoutMessage =
    async (t, promise, message, timeout = 1000) => {
  return Promise.race([
    promise,
    new Promise((resolve, reject) => {
      t.step_timeout(() => {
        reject(new Error(message));
      }, timeout);
    }),
  ]);
}

// Helper method for use with history.back(), when we want to be
// sure that its asynchronous effect has completed.
const waitForUrlToEndWith = async (url) => {
  return new Promise((resolve, reject) => {
    window.addEventListener('popstate', () => {
      if (location.href.endsWith(url)) {
        resolve();
      } else {
        reject(
            'Got ' + location.href + ' - expected URL ends with "' + url + '"');
      }
    }, {once: true});
  });
};

const testSoftNavigation = options => {
  const addContent = options.addContent;
  const link = options.link;
  const pushState =
      readValue(options.pushState, url => {history.pushState({}, '', url)});
  const clicks = readValue(options.clicks, 1);
  const extraValidations = readValue(options.extraValidations, () => {});
  const testName = options.testName;
  const pushUrl = readValue(options.pushUrl, URL);
  const eventType = readValue(options.eventType, 'click');
  const interactionFunc = options.interactionFunc;
  const eventPrepWork = options.eventPrepWork;
  const preTestWork = options.preTestWork;
  promise_test(async t => {
    if (preTestWork) {
      await preTestWork();
    }
    await withTimeoutMessage(
        t, waitInitialLCP(), 'Timed out waiting for initial LCP');
    const preClickLcp = await withTimeoutMessage(
        t, getLcpEntries(), 'Timed out waiting for LCP entries');
    setEvent(t, link, pushState, addContent, pushUrl, eventType, eventPrepWork);
    let first_navigation_id;
    for (let i = 0; i < clicks; ++i) {
      const firstClick = (i === 0);
      let paint_entries_promise = waitOnPaintEntriesPromise(firstClick);
      interacted = false;
      const soft_nav_promise = waitOnSoftNav();
      interact(link, interactionFunc);

      const navigation_id = await withTimeoutMessage(
          t, soft_nav_promise, 'Timed out waiting for soft navigation', 3000);
      if (!first_navigation_id) {
        first_navigation_id = navigation_id;
      }
      // Ensure paint timing entries are fired before moving on to the next
      // click.
      await withTimeoutMessage(
          t, paint_entries_promise, 'Timed out waiting for paint entries');
    }
    assert_equals(
        document.softNavigations, clicks,
        'Soft Navigations detected are the same as the number of clicks');
    await withTimeoutMessage(
        t, validateSoftNavigationEntry(clicks, extraValidations, pushUrl),
        'Timed out waiting for soft navigation entry validation');

    await withTimeoutMessage(
        t,
        runEntryValidations(
            preClickLcp, first_navigation_id, clicks + 1, options.validate),
        'Timed out waiting for entry validations');
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
    const soft_nav_promise = waitOnSoftNav();
    interact(link);
    const first_navigation_id = await soft_nav_promise;
    await navigated;
    await paint_entries_promise;
    assert_equals(document.softNavigations, 1, 'Soft Navigation detected');
    await validateSoftNavigationEntry(1, () => {}, 'foobar.html');

    await runEntryValidations(preClickLcp, first_navigation_id);
  }, testName);
};

const runEntryValidations = async (
    preClickLcp, first_navigation_id, entries_expected_number = 2,
    validate = null) => {
  const postClickLcp = await getLcpEntries();
  const postClickLcpWithoutSoftNavs = await getLcpEntriesWithoutSoftNavs();
  assert_greater_than(
      postClickLcp.length, preClickLcp.length,
      'Soft navigation should have triggered at least an LCP entry');

  if (validate) {
    await validate();
  }
  assert_equals(
      postClickLcpWithoutSoftNavs.length, preClickLcp.length,
      'Soft navigation should not have triggered an LCP entry when the ' +
          'observer did not opt in');
  assert_not_equals(
      postClickLcp[postClickLcp.length - 1].size,
      preClickLcp[preClickLcp.length - 1].size,
      'Soft navigation LCP element should not have identical size to the hard ' +
          'navigation LCP element');
  assert_equals(
      postClickLcp[preClickLcp.length].navigationId, first_navigation_id,
      'Soft navigation LCP should have the same navigation ' +
          'ID as the last soft nav entry');
};

const interact =
    (link, interactionFunc = undefined) => {
      if (test_driver) {
        if (interactionFunc) {
          interactionFunc();
        } else {
          test_driver.click(link);
        }
        timestamps[counter] = {'syncPostInteraction': performance.now()};
      }
    }

const setEvent =
    (t, button, pushState, addContent, pushUrl, eventType, prepWork) => {
      const eventObject =
          (eventType == 'click' || eventType.startsWith('key')) ? button :
                                                                  window;
      eventObject.addEventListener(eventType, async e => {
        let prepWorkFailed = false;
        if (prepWork && !prepWork(t)) {
          prepWorkFailed = true;
        }
        // This is the end of the event's sync processing.
        if (!timestamps[counter]['eventEnd']) {
          timestamps[counter]['eventEnd'] = performance.now();
        }
        if (prepWorkFailed) {
          return;
        }
        // Jump through a task, to ensure task tracking is working properly.
        await new Promise(r => t.step_timeout(r, 0));

        const url = URL + '?' + counter;
        if (pushState) {
          // Change the URL; pushState may be asynchronous, e.g. to deal
          // with history.back()'s asynchronous effect.
          if (pushUrl) {
            await pushState(url);
          } else {
            await pushState();
          }
        }

        // Wait 10 ms to make sure the timestamps are correct.
        await new Promise(r => t.step_timeout(r, 10));

        await addContent(url);

        interacted = true;
        ++counter;
      });
    };

const validateSoftNavigationEntry =
    async (clicks, extraValidations, pushUrl) => {
  const [entries, options] = await new Promise(resolve => {
    new PerformanceObserver((list, obs, options) => {
      resolve([list.getEntries(), options]);
    }).observe({type: 'soft-navigation', buffered: true});
  });
  const expectedClicks = Math.min(clicks, MAX_CLICKS);

  assert_equals(
      entries.length, expectedClicks, 'Performance observer got an entry');
  for (let i = 0; i < entries.length; ++i) {
    const entry = entries[i];
    assert_true(
        entry.name.includes(pushUrl ? pushUrl : document.location.href),
        'The soft navigation name is properly set');
    const entryTimestamp = entry.startTime;
    assert_less_than_equal(
        timestamps[i]['syncPostInteraction'], entryTimestamp,
        'Entry timestamp is lower than the post interaction one');
    assert_greater_than_equal(
        entryTimestamp, timestamps[i]['eventEnd'],
        'Event start timestamp matches');
    assert_not_equals(
        entry.navigationId,
        performance.getEntriesByType('navigation')[0].navigationId,
        'The navigation ID was re-generated and different from the initial one.');
    if (i > 0) {
      assert_not_equals(
          entry.navigationId, entries[i - 1].navigationId,
          'The navigation ID was re-generated between clicks');
    }
  }
  assert_equals(
      performance.getEntriesByType('soft-navigation').length, expectedClicks,
      'Performance timeline got an entry');
  await extraValidations(entries, options);
};

const waitInitialLCP =
    () => {
      return new Promise(resolve => {
        new PerformanceObserver(resolve).observe(
            {type: 'largest-contentful-paint', buffered: true});
      });
    }

const waitOnSoftNav = () => {
  return new Promise(resolve => {
    new PerformanceObserver(list => {
      const entries = list.getEntries();
      assert_equals(entries.length, 1, 'Only one soft navigation entry');
      resolve(entries[0].navigationId);
    }).observe({type: 'soft-navigation'});
  });
};

const getLcpEntries = async () => {
  const entries = await new Promise(resolve => {
    new PerformanceObserver(list => {
      resolve(list.getEntries());
    }).observe({
      type: 'largest-contentful-paint',
      buffered: true,
      includeSoftNavigationObservations: true
    });
  });
  return entries;
};

const getLcpEntriesWithoutSoftNavs = async () => {
  const entries = await new Promise(resolve => {
    new PerformanceObserver(list => {
      resolve(list.getEntries());
    }).observe({type: 'largest-contentful-paint', buffered: true});
  });
  return entries;
};

const addImage = async (element, url = 'blue.png', id = 'imagelcp') => {
  const img = new Image();
  img.src = '/images/' + url + '?' + Math.random();
  img.id = id;
  img.setAttribute('elementtiming', id);
  await img.decode();
  element.appendChild(img);
};
const addImageToMain = async (url = 'blue.png', id = 'imagelcp') => {
  await addImage(document.getElementById('main'), url, id);
};

const addTextParagraphToMain = (text, element_timing = '') => {
  const main = document.getElementById('main');
  const p = document.createElement('p');
  const textNode = document.createTextNode(text);
  p.appendChild(textNode);
  if (element_timing) {
    p.setAttribute('elementtiming', element_timing);
  }
  p.style = 'font-size: 3em';
  main.appendChild(p);
  return p;
};
const addTextToDivOnMain = () => {
  const main = document.getElementById('main');
  const prevDiv = document.getElementsByTagName('div')[0];
  if (prevDiv) {
    main.removeChild(prevDiv);
  }
  const div = document.createElement('div');
  const text = document.createTextNode('Lorem Ipsum');
  div.appendChild(text);
  div.style = 'font-size: 3em';
  main.appendChild(div);
};

const waitOnPaintEntriesPromise = (expectLCP = true) => {
  return new Promise((resolve, reject) => {
    if (performance.softNavPaintMetricsSupported) {
      const paint_entries = [];
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
      step_timeout(() => {
        requestAnimationFrame(() => {
          requestAnimationFrame(resolve);
        });
      }, 100);
    }
  });
};
