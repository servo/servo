// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: Object.values does not include Symbol keys.
author: Jordan Harband
features: [Symbol]
---*/

var value = {};
var enumSym = Symbol('enum');
var nonEnumSym = Symbol('nonenum');
var symValue = Symbol('value');

var obj = {
  key: symValue
};
obj[enumSym] = value;
Object.defineProperty(obj, nonEnumSym, {
  enumerable: false,
  value: value
});

var result = Object.values(obj);

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 1, 'result has 1 item');

assert.sameValue(result[0], symValue, 'first value is `symValue`');
