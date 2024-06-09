// This is a helper file used for the automatic-beacon-*.https.html tests.
// To use this, make sure you import these scripts:
// <script src="/resources/testharness.js"></script>
// <script src="/resources/testharnessreport.js"></script>
// <script src="/common/utils.js"></script>
// <script src="/common/dispatcher/dispatcher.js"></script>
// <script src="resources/utils.js"></script>
// <script src="/resources/testdriver.js"></script>
// <script src="/resources/testdriver-actions.js"></script>
// <script src="/resources/testdriver-vendor.js"></script>
// <script src="/common/get-host-info.sub.js"></script>

const NavigationTrigger = {
  Click: 0,
  ClickOnce: 1,
  CrossOriginClick: 2,
  CrossOriginClickNoOptIn: 3
};

// Registers an automatic beacon in a given remote context frame, and registers
// the navigation handler for the frame that will trigger the beacon.
//     remote_context: The context for the fenced frame or URN iframe.
//      beacon_events: An array of FenceEvents to register with the frame.
//     navigation_url: The URL the frame will navigate to.
// navigation_trigger: How the navigation will be performed. Either through a
//                     click, a click with a `once` event, a click in a
//                     cross-origin subframe, or a click in a cross-origin
//                     subframe with no opt-in header.
//             target: the target of the navigation. Either '_blank' or
//                     '_unfencedTop'.
async function setupAutomaticBeacon(
    remote_context, beacon_events, navigation_url = 'resources/dummy.html',
    navigation_trigger = NavigationTrigger.Click, target = '_blank') {
  const full_url = new URL(navigation_url, location.href);
  await remote_context.execute(
      async (
          NavigationTrigger, beacon_events, navigation_trigger, full_url,
          target) => {
        switch (navigation_trigger) {
          case NavigationTrigger.Click:
            addEventListener('click', (event) => {
              beacon_events.forEach((beacon_event) => {
                window.fence.setReportEventDataForAutomaticBeacons(
                    beacon_event);
              });
              window.open(full_url, target);
            });
            break;
          case NavigationTrigger.ClickOnce:
            beacon_events.forEach((beacon_event) => {
              window.fence.setReportEventDataForAutomaticBeacons(beacon_event);
            });
            addEventListener('click', (event) => {
              window.open(full_url, target);
            });
            break;
          case NavigationTrigger.CrossOriginClick:
          case NavigationTrigger.CrossOriginClickNoOptIn:
            beacon_events.forEach((beacon_event) => {
              window.fence.setReportEventDataForAutomaticBeacons(beacon_event);
            });
            // Add a cross-origin iframe that will perform the top-level
            // navigation. Do not set the 'Allow-Fenced-Frame-Automatic-Beacons'
            // header to true.
            const iframe = await attachIFrameContext({
              origin: get_host_info().HTTPS_REMOTE_ORIGIN,
              headers: [[
                'Allow-Fenced-Frame-Automatic-Beacons',
                navigation_trigger == NavigationTrigger.CrossOriginClick ?
                    'true' :
                    'false'
              ]]
            });
            await iframe.execute(async (full_url, target) => {
              addEventListener('click', (event) => {
                window.open(full_url, target);
              });
            }, [full_url, target]);
            break;
        }
      },
      [NavigationTrigger, beacon_events, navigation_trigger, full_url, target]);
}

// Checks if an automatic beacon of type `event_type` with contents `event_data`
// was sent out or not.
//        event_type: The automatic beacon type to check.
//        event_data: The automatic beacon data to check.
// expected_referrer: The expected referrer header, if different from origin.
//  expected_success: Whether we expect the automatic beacon to be sent.
//                 t: The WPT's test object. Only required if
//                   expected_success = false.
async function verifyBeaconData(
    event_type, event_data, expected_referrer = null, expected_success = true,
    t) {
  if (expected_success) {
    const data = await nextBeacon(event_type, event_data);
    const [beacon_initiator_origin, beacon_referrer] =
        data.split(",");
    assert_equals(beacon_initiator_origin, get_host_info().HTTPS_ORIGIN,
        "The initiator origin should be set as expected.");
    // The Referer header has a trailing '/' appended to the URL.
    assert_equals(beacon_referrer,
        (expected_referrer ? expected_referrer :
        get_host_info().HTTPS_ORIGIN) + "/",
        "The beacon referrer should be set as expected.");
  } else {
    const timeout = new Promise(r => t.step_timeout(r, 1000));
    const result =
        await Promise.race([nextBeacon(event_type, event_data), timeout]);
    assert_true(typeof result === 'undefined',
        "The beacon should not have sent.");
  }
}
