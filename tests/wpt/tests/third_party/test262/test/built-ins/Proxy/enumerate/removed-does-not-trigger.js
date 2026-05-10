// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots
description: >
  Enumerate trap was removed and it should not be triggered anymore.
includes: [compareArray.js]
features: [Proxy, Symbol, Symbol.iterator]
---*/

var x;
var target = [1, 2, 3];
var p = new Proxy(target, {
  enumerate: function() {
    throw new Test262Error(
      "An enumerate property on handler object shouldn't trigger a Proxy trap"
    );
  }
});

var forInResults = [];
for (x in p) {
  forInResults.push(x);
}

assert.compareArray(forInResults, ["0", "1", "2"]);

var forOfResults = [];
for (x of p) {
  forOfResults.push(x);
}

assert.compareArray(forOfResults, [1, 2, 3]);

var itor = p[Symbol.iterator]();
var next = itor.next();
assert.sameValue(next.value, 1);
assert.sameValue(next.done, false);
next = itor.next();
assert.sameValue(next.value, 2);
assert.sameValue(next.done, false);
next = itor.next();
assert.sameValue(next.value, 3);
assert.sameValue(next.done, false);
next = itor.next();
assert.sameValue(next.value, undefined);
assert.sameValue(next.done, true);
