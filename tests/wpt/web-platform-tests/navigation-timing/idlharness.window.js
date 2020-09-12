// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// http://www.w3.org/TR/navigation-timing/

idl_test(
  ['navigation-timing', 'hr-time'],
  ['resource-timing', 'performance-timeline', 'dom', 'html'],
  idlArray => {
    idlArray.add_objects({
      Performance: ['performance'],
      PerformanceNavigation: ['performance.navigation'],
      PerformanceTiming: ['performance.timing'],
      PerformanceNavigationTiming: ['performance.getEntriesByType("navigation")[0]']
    });
  }
);
