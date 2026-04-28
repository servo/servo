// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol-constructor
description: The first argument is coerced to a String value (from an object)
info: |
    1. If NewTarget is not undefined, throw a TypeError exception.
    2. If description is undefined, let descString be undefined.
    2. Else, let descString be ? ToString(description).
    3. Return a new unique Symbol value whose [[Description]] value is
       descString.
features: [Symbol]
---*/

var calls, val;

val = {
  toString: function() {
    calls += 'toString';
    return {};
  },
  valueOf: function() {
    calls += 'valueOf';
  }
};

calls = '';
Symbol(val);
assert.sameValue(calls, 'toStringvalueOf');

val = {
  toString: function() {
    calls += 'toString';
  },
  valueOf: function() {
    calls += 'valueOf';
  }
};

calls = '';
Symbol(val);
assert.sameValue(calls, 'toString');

val = {
  toString: null,
  valueOf: function() {
    calls += 'valueOf';
  }
};

calls = '';
Symbol(val);
assert.sameValue(calls, 'valueOf');

val = {
  toString: null,
  valueOf: function() {
    calls += 'valueOf';
    return {};
  }
};

calls = '';
assert.throws(TypeError, function() {
  Symbol(val);
}, '`toString` is not callable, and `valueOf` returns a non-primitive value');
assert.sameValue(
  calls, 'valueOf', 'invocation pattern for non-callable `toString`'
);

val = {
  toString: function() {
    calls += 'toString';
    return {};
  },
  valueOf: function() {
    calls += 'valueOf';
    return {};
  }
};

calls = '';
assert.throws(TypeError, function() {
  Symbol(val);
}, '`toString` nor `valueOf` both return non-primitive values');
assert.sameValue(
  calls,
  'toStringvalueOf',
  'invocation pattern for non-callable `toString` and `valueOf`'
);
