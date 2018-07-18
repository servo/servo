// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// http://www.w3.org/TR/navigation-timing/

idl_test(
  ['navigation-timing'],
  ['resource-timing', 'performance-timeline', 'hr-time', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Performance: ['performance'],
      PerformanceNavigation: ['performance.navigation'],
      PerformanceTiming: ['performance.timing'],
      PerformanceNavigationTiming: [
        'performance.getEntriesByType("navigation")[0]'
      ]
    });
  },
  'navigation-timing interfaces'
);
