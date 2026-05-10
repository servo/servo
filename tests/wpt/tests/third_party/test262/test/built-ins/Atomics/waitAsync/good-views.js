// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Test Atomics.waitAsync on arrays that allow atomic operations
flags: [async]
includes: [atomicsHelper.js, asyncHelpers.js]
features: [Atomics.waitAsync, Atomics]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

$262.agent.start(`
  (async () => {
    var sab = new SharedArrayBuffer(1024);
    var good_indices = [ (view) => 0/-1, // -0
                         (view) => '-0',
                         (view) => view.length - 1,
                         (view) => ({ valueOf: () => 0 }),
                         (view) => ({ toString: () => '0', valueOf: false }) // non-callable valueOf triggers invocation of toString
                       ];

    var view = new Int32Array(sab, 32, 20);

    view[0] = 0;
    $262.agent.report("A " + (await Atomics.waitAsync(view, 0, 0, 0).value))
    $262.agent.report("B " + (await Atomics.waitAsync(view, 0, 37, 0).value));

    const results = [];
    // In-bounds boundary cases for indexing
    for ( let IdxGen of good_indices ) {
        let Idx = IdxGen(view);
        view.fill(0);
        // Atomics.store() computes an index from Idx in the same way as other
        // Atomics operations, not quite like view[Idx].
        Atomics.store(view, Idx, 37);
        results.push(await Atomics.waitAsync(view, Idx, 0).value);
    }
    $262.agent.report("C " + results.join(","));
    $262.agent.leaving();
  })();
`);


asyncTest(async () => {
  const outcomes = [];

  for (let i = 0; i < 3; i++) {
    outcomes.push(await $262.agent.getReportAsync());
  }

  assert.sameValue(
    outcomes[0],
    'A timed-out',
    'The value of outcomes[0] is "A timed-out"'
  );

  assert.sameValue(
    outcomes[1],
    'B not-equal',
    'The value of outcomes[1] is "B not-equal"'
  );
  assert.sameValue(
    outcomes[2],
    'C not-equal,not-equal,not-equal,not-equal,not-equal',
    'The value of outcomes[2] is "C not-equal,not-equal,not-equal,not-equal,not-equal"'
  );
});
