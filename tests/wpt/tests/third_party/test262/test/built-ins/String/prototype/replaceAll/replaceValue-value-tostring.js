// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  ToString(replaceValue)
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  5. Let functionalReplace be IsCallable(replaceValue).
  6. If functionalReplace is false, then
    a. Let replaceValue be ? ToString(replaceValue). 
  ...
  14. For each position in matchPositions, do
    a. If functionalReplace is true, then
      ...
    b. Else,
      ...
      ii. Let captures be a new empty List.
      iii. Let replacement be GetSubstitution(searchString, string, position, captures, undefined, replaceValue).
features: [String.prototype.replaceAll, Symbol.toPrimitive]
---*/

var result;

var called;
var replaceValue;

called = 0;
replaceValue = {
  [Symbol.toPrimitive](){
    called += 1;
    return 'z';
  },
  toString() {
    throw 'poison';
  },
  valueOf() {
    throw 'poison';
  },
};

result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'zz', 'object @@toPrimitive');
assert.sameValue(called, 1, '@@toPrimitive is called only once');

called = 0;
replaceValue = {
  [Symbol.toPrimitive]: undefined,
  toString() {
    called += 1;
    return 'z';
  },
  valueOf() {
    throw 'poison';
  },
};

result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'zz', 'object toString');
assert.sameValue(called, 1, 'toString is called only once');

called = 0;
replaceValue = {
  [Symbol.toPrimitive]: undefined,
  toString: undefined,
  valueOf() {
    called += 1;
    return 'z';
  },
};

result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'zz', 'object valueOf');
assert.sameValue(called, 1, 'valueOf is called only once');

replaceValue = 42;
result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, '4242', 'number');

replaceValue = true;
result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'truetrue', 'Boolean true');

replaceValue = false;
result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'falsefalse', 'Boolean false');

replaceValue = undefined;
result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'undefinedundefined', 'undefined');

replaceValue = null;
result = 'aa'.replaceAll('a', replaceValue);
assert.sameValue(result, 'nullnull', 'null');
