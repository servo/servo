// META: global=window,dedicatedworker,sharedworker,serviceworker
'use strict';

promise_test(async testCase => {
  assert_class_string(navigator.screen, 'ScreenManager');
  assert_equals(typeof navigator.screen.getScreens, 'function');
}, 'navigator.screen.getScreens is present');

promise_test(async testCase => {
  const screens = await navigator.screen.getScreens();
  assert_greater_than(screens.length, 0);

  assert_equals(typeof screens[0].availWidth, 'number');
  assert_equals(typeof screens[0].availHeight, 'number');
  assert_equals(typeof screens[0].width, 'number');
  assert_equals(typeof screens[0].height, 'number');
  assert_equals(typeof screens[0].colorDepth, 'number');
  assert_equals(typeof screens[0].pixelDepth, 'number');

  assert_equals(typeof screens[0].availLeft, 'number');
  assert_equals(typeof screens[0].availTop, 'number');
  assert_equals(typeof screens[0].left, 'number');
  assert_equals(typeof screens[0].top, 'number');
  assert_equals(typeof screens[0].orientation, 'object');

  assert_equals(typeof screens[0].primary, 'boolean');
  assert_equals(typeof screens[0].internal, 'boolean');
  assert_equals(typeof screens[0].scaleFactor, 'number');
  assert_equals(typeof screens[0].name, 'string');
}, 'navigator.screen.getScreens returns at least 1 Screen');