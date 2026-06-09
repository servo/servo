// PerformanceObserver.supportedEntryTypes is a good way to detect whether
// scroll performance timing is supported.
// https://github.com/MicrosoftEdge/MSEdgeExplainers/blob/main/PerformanceScrollTiming/explainer.md

test(() => {
  assert_in_array('scroll', PerformanceObserver.supportedEntryTypes);
}, '"scroll" is a supported entry type for PerformanceObserver');

test(() => {
  assert_implements(window.PerformanceScrollTiming,
                    'PerformanceScrollTiming is not implemented');
}, 'PerformanceScrollTiming interface is exposed on window');
