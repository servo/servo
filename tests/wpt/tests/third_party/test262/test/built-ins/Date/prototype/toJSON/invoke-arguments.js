// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  toISOString is called with correct context and without arguments.
info: |
  Date.prototype.toJSON ( key )

  [...]
  4. Return ? Invoke(O, "toISOString").

  Invoke ( V, P [ , argumentsList ] )

  [...]
  3. Let func be ? GetV(V, P).
  4. Return ? Call(func, V, argumentsList).
---*/

var getCount = 0, getContext;
var callCount = 0, callContext, callArguments;
var obj = {
  get toISOString() {
    getCount += 1;
    getContext = this;

    return function() {
      callCount += 1;
      callContext = this;
      callArguments = arguments;
    };
  },
};

Date.prototype.toJSON.call(obj);

assert.sameValue(getCount, 1);
assert.sameValue(getContext, obj);

assert.sameValue(callCount, 1);
assert.sameValue(callContext, obj);
assert.sameValue(callArguments.length, 0);
