// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors includes Symbol keys.
esid: sec-object.getownpropertydescriptors
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
  writable: true,
  enumerable: false,
  configurable: true,
  value: value
});

var result = Object.getOwnPropertyDescriptors(obj);

assert.sameValue(Object.keys(result).length, 1, 'obj has 1 string-keyed descriptor');
assert.sameValue(Object.getOwnPropertySymbols(result).length, 2, 'obj has 2 symbol-keyed descriptors');

assert.sameValue(result.key.configurable, true, 'result.key is configurable');
assert.sameValue(result.key.enumerable, true, 'result.key is enumerable');
assert.sameValue(result.key.writable, true, 'result.key is writable');
assert.sameValue(result.key.value, symValue, 'result.key has value symValue');

assert.sameValue(result[enumSym].configurable, true, 'result[enumSym] is configurable');
assert.sameValue(result[enumSym].enumerable, true, 'result[enumSym] is enumerable');
assert.sameValue(result[enumSym].writable, true, 'result[enumSym] is writable');
assert.sameValue(result[enumSym].value, value, 'result[enumSym] has value `value`');

assert.sameValue(result[nonEnumSym].configurable, true, 'result[nonEnumSym] is configurable');
assert.sameValue(result[nonEnumSym].enumerable, false, 'result[nonEnumSym] is not enumerable');
assert.sameValue(result[nonEnumSym].writable, true, 'result[nonEnumSym] is writable');
assert.sameValue(result[nonEnumSym].value, value, 'result[nonEnumSym] has value `value`');
