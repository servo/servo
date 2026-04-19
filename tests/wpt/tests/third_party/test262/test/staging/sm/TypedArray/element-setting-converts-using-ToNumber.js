/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  Typed array element-setting should convert to target type using ToNumber followed by an element-type-specific truncation function
info: bugzilla.mozilla.org/show_bug.cgi?id=985733
esid: pending
---*/

anyTypedArrayConstructors.forEach(function(TypedArray) {
  var ta = new TypedArray(1);
  assert.sameValue(ta[0], 0);

  var count = 0;
  function setToObject()
  {
    for (var i = 0; i < 1e4; i++)
    {
      assert.sameValue(count, i);
      ta[0] = { valueOf: function() { count++; return 17; } };
    }
  }
  setToObject();
  assert.sameValue(count, 1e4);
  assert.sameValue(ta[0], 17);

  function setToString()
  {
    for (var i = 0; i < 2e4; i++)
      ta[0] = "17.0000000000000000000000000000000000000000000000000000001";
  }
  setToString();
  assert.sameValue(ta[0], 17);

  count = 0;
  var arrayConstructed =
    new TypedArray([{ valueOf: function() { count++; return 17; } },
                   "17.0000000000000000000000000000000000000000000000000001"]);
  assert.sameValue(count, 1);
  assert.sameValue(arrayConstructed[0], 17);
  assert.sameValue(arrayConstructed[1], 17);

  count = 0;
  var arraySet = new TypedArray(5);
  arraySet.set({ 0: 17,
                 1: "17.000000000000000000000000000000000000000000000000000",
                 get 2() {
                   return { valueOf: undefined,
                            toString: function() { count++; return 42; } };
                 },
                 get 3() { return true; },
                 set 3(v) { throw "FAIL"; },
                 4: { valueOf: function() { count++; return 127; } },
                 length: 5 });
  assert.sameValue(count, 2);
  assert.sameValue(arraySet[0], 17);
  assert.sameValue(arraySet[1], 17);
  assert.sameValue(arraySet[2], 42);
  assert.sameValue(arraySet[3], 1);
  assert.sameValue(arraySet[4], 127);

  var bigLen = 1e4;
  var big = new TypedArray(bigLen);
  function initBig()
  {
    for (var i = 0; i < bigLen; i++)
      big[i] = (i % 2) ? 3 : { valueOf: function() { return 3; } };
  }
  initBig();
  for (var i = 0; i < bigLen; i++)
  {
    assert.sameValue(big[i], 3,
             "(" + Object.prototype.toString.call(big) + ")");
  }
});
