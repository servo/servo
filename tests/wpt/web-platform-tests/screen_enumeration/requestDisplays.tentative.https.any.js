// META: global=window,dedicatedworker,sharedworker,serviceworker
'use strict';

promise_test(async testCase => {
  assert_class_string(navigator.screen, 'ScreenManager');
  assert_equals(typeof navigator.screen.requestDisplays, 'function');
}, 'navigator.screen.requestDisplays is present');

promise_test(async testCase => {
  const displays = await navigator.screen.requestDisplays();
  assert_greater_than(displays.length, 0);
  assert_equals(typeof displays[0].name, 'string');
  assert_equals(typeof displays[0].scaleFactor, 'number');
  assert_equals(typeof displays[0].width, 'number');
  assert_equals(typeof displays[0].height, 'number');
  assert_equals(typeof displays[0].left, 'number');
  assert_equals(typeof displays[0].top, 'number');
  assert_equals(typeof displays[0].colorDepth, 'number');
  assert_equals(typeof displays[0].isPrimary, 'boolean');
  assert_equals(typeof displays[0].isInternal, 'boolean');
}, 'navigator.screen.requestDisplays returns at least 1 Display');