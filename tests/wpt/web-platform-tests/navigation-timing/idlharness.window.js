// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// http://www.w3.org/TR/navigation-timing/

promise_test(async (t) => {
  const srcs = ['navigation-timing', 'hr-time', 'resource-timing', 'performance-timeline', 'dom', 'html'];
  const [navigationTiming, hrTime, resourceTiming, performanceTimeline, dom, html] =
      await Promise.all(srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));
  const idlArray = new IdlArray();
  idlArray.add_idls(hrTime);
  idlArray.add_idls(navigationTiming);
  idlArray.add_dependency_idls(resourceTiming);
  idlArray.add_dependency_idls(performanceTimeline);
  idlArray.add_dependency_idls(dom);
  idlArray.add_dependency_idls(html);
  idlArray.add_objects({
    Performance: ['performance'],
    PerformanceNavigation: ['performance.navigation'],
    PerformanceTiming: ['performance.timing'],
    PerformanceNavigationTiming: [
      'performance.getEntriesByType("navigation")[0]'
    ]
  });
  return idlArray.test();
})
