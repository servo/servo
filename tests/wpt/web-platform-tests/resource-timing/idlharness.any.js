// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/resource-timing/

promise_test(async () => {
  const [idl, perf, hrtime, dom, html] = await Promise.all([
    '/interfaces/resource-timing.idl',
    '/interfaces/performance-timeline.idl',
    '/interfaces/hr-time.idl',
    '/interfaces/dom.idl',
    '/interfaces/html.idl',
  ].map(url => fetch(url).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(perf);
  idl_array.add_dependency_idls(hrtime);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);
  idl_array.add_objects({
    Performance: ['performance'],
    PerformanceResourceTiming: ["performance.getEntriesByType('resource')[0]"]
  });
  idl_array.test();
}, 'Test server-timing IDL implementation');
