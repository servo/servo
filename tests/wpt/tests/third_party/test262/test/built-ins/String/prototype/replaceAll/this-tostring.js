// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  ToString(this value)
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  3. Let string be ? ToString(O).
  4. Let searchString be ? ToString(searchValue).
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
var thisValue;

called = 0;
thisValue = {
  [Symbol.toPrimitive](){
    called += 1;
    return 'aa';
  },
  toString() {
    throw 'poison';
  },
  valueOf() {
    throw 'poison';
  },
};

result = ''.replaceAll.call(thisValue, 'a', 'z');
assert.sameValue(result, 'zz', 'object @@toPrimitive');
assert.sameValue(called, 1, '@@toPrimitive is called only once');

called = 0;
thisValue = {
  [Symbol.toPrimitive]: undefined,
  toString() {
    called += 1;
    return 'aa';
  },
  valueOf() {
    throw 'poison';
  },
};

result = ''.replaceAll.call(thisValue, 'a', 'z');
assert.sameValue(result, 'zz', 'object toString');
assert.sameValue(called, 1, 'toString is called only once');

called = 0;
thisValue = {
  [Symbol.toPrimitive]: undefined,
  toString: undefined,
  valueOf() {
    called += 1;
    return 'aa';
  },
};

result = ''.replaceAll.call(thisValue, 'a', 'z');
assert.sameValue(result, 'zz', 'object valueOf');
assert.sameValue(called, 1, 'valueOf is called only once');

thisValue = 4244;
result = ''.replaceAll.call(thisValue, '4', 'z');
assert.sameValue(result, 'z2zz', 'number');

thisValue = true;
result = ''.replaceAll.call(thisValue, 'ru', 'o m');
assert.sameValue(result, 'to me', 'Boolean true');

thisValue = false;
result = ''.replaceAll.call(thisValue, 'al', 'on');
assert.sameValue(result, 'fonse', 'Boolean false');
