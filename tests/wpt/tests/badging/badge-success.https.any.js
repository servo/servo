"use strict";

test(() => {
  assert_true("setAppBadge" in navigator, "navigator.setAppBadge should exist");
  assert_true(
    "clearAppBadge" in navigator,
    "navigator.clearAppBadge should exist",
  );
}, "Badging API interface exists");

promise_test(async () => {
  const numberTestCases = [
    { value: undefined, desc: "undefined" },
    { value: null, desc: "null (coerced to 0)" },
    { value: 1, desc: "integer value of 1" },
    { value: 10.6, desc: "non-whole number" },
    { value: Number.MAX_SAFE_INTEGER, desc: "maximum allowed value" },
    { value: 0, desc: "zero" },
  ];

  for (const { value, desc } of numberTestCases) {
    const result = await navigator.setAppBadge(value);
    assert_equals(
      result,
      undefined,
      `setAppBadge resolves successfully when passed ${desc} as input`,
    );
  }
}, "Resolves successfully for number input cases");

promise_test(async () => {
  const stringTestCases = [
    { value: "3", desc: "numeric string '3' (coerced to 3)" },
    {
      value: " 300.000 ",
      desc: "numeric string ' 300.000 ' (coerced to 300)",
    },
    { value: "", desc: "empty string (coerced to 0)" },
  ];

  for (const { value, desc } of stringTestCases) {
    const result = await navigator.setAppBadge(value);
    assert_equals(
      result,
      undefined,
      `setAppBadge resolves successfully when passed ${desc} as input`,
    );
  }
}, "Resolves successfully for string input cases");

promise_test(async () => {
  const resultFalse = await navigator.setAppBadge(false);
  assert_equals(
    resultFalse,
    undefined,
    "setAppBadge resolves successfully when passed false as input (coerced to 0)",
  );

  const resultTrue = await navigator.setAppBadge(true);
  assert_equals(
    resultTrue,
    undefined,
    "setAppBadge resolves successfully when passed true as input (coerced to 1)",
  );
}, "Resolves successfully for boolean input cases");

promise_test(async () => {
  const result = await navigator.clearAppBadge();
  assert_equals(result, undefined, "clearAppBadge should return undefined");
}, "clearAppBadge resolves successfully");

promise_test(async () => {
  // Test calling setAppBadge with no arguments (should set flag)
  const result = await navigator.setAppBadge();
  assert_equals(
    result,
    undefined,
    "setAppBadge() with no arguments should return undefined",
  );
}, "setAppBadge with no arguments succeeds (flag mode)");
