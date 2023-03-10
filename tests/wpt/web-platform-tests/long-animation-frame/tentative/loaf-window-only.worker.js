importScripts("/resources/testharness.js");

test(() => {
  assert_false(PerformanceObserver.supportedEntryTypes.includes("long-animation-frame"));
}, 'PerformanceObserver should not include "long-animation-frame" in workers');

test(() => {
  assert_false("PerformanceLongAnimationFrameTiming" in self);
}, 'PerformanceLongAnimationFrameTiming should not be exposed in workers');

done();
