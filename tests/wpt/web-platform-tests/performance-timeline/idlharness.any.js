// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/performance-timeline/

'use strict';

promise_test(async () => {
  const idl_array = new IdlArray();
  const idl = await fetch("/interfaces/performance-timeline.idl").then(r => r.text());
  const dom = await fetch("/interfaces/dom.idl").then(r => r.text());
  const hrtime = await fetch("/interfaces/hr-time.idl").then(r => r.text());

  // create first mark
  self.performance.mark("mark");

  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(hrtime);
  idl_array.add_dependency_idls(dom);
  idl_array.add_objects({
    Performance: ["performance"],
    PerformanceMark: [self.performance.getEntriesByName("mark")[0]],
  });
  idl_array.test();
}, "Test IDL implementation of performance-timeline API");
