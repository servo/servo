// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  Abrupt completion from GetV or Call.
info: |
  Date.prototype.toJSON ( key )

  [...]
  4. Return ? Invoke(O, "toISOString").

  Invoke ( V, P [ , argumentsList ] )

  [...]
  3. Let func be ? GetV(V, P).
  4. Return ? Call(func, V, argumentsList).
---*/

var abruptGet = {
  get toISOString() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  Date.prototype.toJSON.call(abruptGet);
});

var abruptCall = {
  toISOString() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  Date.prototype.toJSON.call(abruptCall);
});
