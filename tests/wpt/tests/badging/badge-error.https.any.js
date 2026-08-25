"use strict";

promise_test(async (t) => {
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge(-1),
    "Reject with TypeError if the value is negative",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge(Number.MAX_SAFE_INTEGER + 1),
    "Reject with TypeError if the value is larger than the maximum safe integer (2^53 - 1)",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge(Infinity),
    "Reject with TypeError if the value is positive infinity",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge(-Infinity),
    "Reject with TypeError if the value is negative infinity",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge(NaN),
    "Reject with TypeError if the value is NaN",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge("Foo"),
    "Reject with TypeError if the value cannot be converted to a long: string",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge({}),
    "Reject with TypeError if the value cannot be converted to a long: object",
  );
  await promise_rejects_js(
    t,
    TypeError,
    navigator.setAppBadge([]),
    "Reject with TypeError if the value cannot be converted to a long: array",
  );
}, "Test various invalid input cases for setAppBadge()");

promise_test(async () => {
  // Test sequential operations don't interfere
  await navigator.setAppBadge(1);
  await navigator.setAppBadge(5);
  await navigator.setAppBadge(0);
  await navigator.clearAppBadge();
  await navigator.setAppBadge();
  await navigator.clearAppBadge();
}, "Sequential badge operations work correctly");
