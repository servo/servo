// PerformanceObserver.supportedEntryTypes is a good way to detect whether
// soft navigations are supported. See also:
// https://developer.mozilla.org/en-US/docs/Web/API/PerformanceObserver/supportedEntryTypes_static

test(() => {
  assert_in_array('soft-navigation', PerformanceObserver.supportedEntryTypes);
}, 'Soft navigations are a supported entry type for PerformanceObserver');
