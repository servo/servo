/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Behavior when the JSON.parse reviver mutates the holder array
info: bugzilla.mozilla.org/show_bug.cgi?id=901351
esid: pending
---*/

var proxyObj = null;

var arr = JSON.parse('[0, 1]', function(prop, v) {
  if (prop === "0")
  {
    proxyObj = new Proxy({ c: 17, d: 42 }, {});
    this[1] = proxyObj;
  }
  return v;
});

assert.sameValue(arr[0], 0);
assert.sameValue(arr[1], proxyObj);
assert.sameValue(arr[1].c, 17);
assert.sameValue(arr[1].d, 42);
