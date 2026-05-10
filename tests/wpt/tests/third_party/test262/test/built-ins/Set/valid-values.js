// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.add
description: Observing the expected behavior of valid values
info: |
  Set.prototype.add ( value )

  ...
  For each element e of entries, do
    If e is not empty and SameValueZero(e, value) is true, then
    Return S.
  If value is -0, set value to +0.
  Append value as the last element of entries.
  ...

features: [BigInt, Symbol, TypedArray, WeakRef, exponentiation]
---*/


const negativeZero = -0;
const positiveZero = +0;
const zero = 0;
const one = 1;
const twoRaisedToFiftyThreeMinusOne = 2 ** 53 - 1;
const int32Array = new Int32Array([zero, one]);
const uint32Array = new Uint32Array([zero, one]);
const n = 100000000000000000000000000000000000000000000000000000000000000000000000000000000001n;
const bigInt = BigInt('100000000000000000000000000000000000000000000000000000000000000000000000000000000001');
const n1 = 1n;
const n53 = 9007199254740991n;
const fiftyThree = BigInt('9007199254740991');
const bigInt64Array = new BigInt64Array([n1, n53]);
const bigUint64Array = new BigUint64Array([n1, n53]);
const symbol = Symbol('');
const object = {};
const array = [];
const string = '';
const booleanTrue = true;
const booleanFalse = true;
const functionExprValue = function() {};
const arrowFunctionValue = () => {};
const classValue = class {};
const map = new Map();
const set = new Set();
const weakMap = new WeakMap();
const weakRef = new WeakRef({});
const weakSet = new WeakSet();
const nullValue = null;
const undefinedValue = undefined;
let unassigned;

{
  const s = new Set([negativeZero, negativeZero]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(negativeZero), true);
  s.delete(negativeZero);
  assert.sameValue(s.size, 0);
  s.add(negativeZero);
  assert.sameValue(s.has(negativeZero), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([positiveZero, positiveZero]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(positiveZero), true);
  s.delete(positiveZero);
  assert.sameValue(s.size, 0);
  s.add(positiveZero);
  assert.sameValue(s.has(positiveZero), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([zero, zero]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(zero), true);
  s.delete(zero);
  assert.sameValue(s.size, 0);
  s.add(zero);
  assert.sameValue(s.has(zero), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([one, one]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(one), true);
  s.delete(one);
  assert.sameValue(s.size, 0);
  s.add(one);
  assert.sameValue(s.has(one), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([twoRaisedToFiftyThreeMinusOne, twoRaisedToFiftyThreeMinusOne]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(twoRaisedToFiftyThreeMinusOne), true);
  s.delete(twoRaisedToFiftyThreeMinusOne);
  assert.sameValue(s.size, 0);
  s.add(twoRaisedToFiftyThreeMinusOne);  assert.sameValue(s.has(twoRaisedToFiftyThreeMinusOne), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([int32Array, int32Array]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(int32Array), true);
  s.delete(int32Array);
  assert.sameValue(s.size, 0);
  s.add(int32Array);
  assert.sameValue(s.has(int32Array), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([uint32Array, uint32Array]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(uint32Array), true);
  s.delete(uint32Array);
  assert.sameValue(s.size, 0);
  s.add(uint32Array);
  assert.sameValue(s.has(uint32Array), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([n, n]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(n), true);
  s.delete(n);
  assert.sameValue(s.size, 0);
  s.add(n);
  assert.sameValue(s.has(n), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([bigInt, bigInt]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(bigInt), true);
  s.delete(bigInt);
  assert.sameValue(s.size, 0);
  s.add(bigInt);
  assert.sameValue(s.has(bigInt), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([n1, n1]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(n1), true);
  s.delete(n1);
  assert.sameValue(s.size, 0);
  s.add(n1);
  assert.sameValue(s.has(n1), true);
}
{  const s = new Set([n53, n53]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(n53), true);
  s.delete(n53);
  assert.sameValue(s.size, 0);
  s.add(n53);
  assert.sameValue(s.has(n53), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([fiftyThree, fiftyThree]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(fiftyThree), true);
  s.delete(fiftyThree);
  assert.sameValue(s.size, 0);
  s.add(fiftyThree);
  assert.sameValue(s.has(fiftyThree), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([bigInt64Array, bigInt64Array]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(bigInt64Array), true);
  s.delete(bigInt64Array);
  assert.sameValue(s.size, 0);
  s.add(bigInt64Array);
  assert.sameValue(s.has(bigInt64Array), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([bigUint64Array, bigUint64Array]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(bigUint64Array), true);
  s.delete(bigUint64Array);
  assert.sameValue(s.size, 0);
  s.add(bigUint64Array);
  assert.sameValue(s.has(bigUint64Array), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([symbol, symbol]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(symbol), true);
  s.delete(symbol);
  assert.sameValue(s.size, 0);
  s.add(symbol);
  assert.sameValue(s.has(symbol), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([object, object]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(object), true);
  s.delete(object);
  assert.sameValue(s.size, 0);
  s.add(object);
  assert.sameValue(s.has(object), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([array, array]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(array), true);
  s.delete(array);
  assert.sameValue(s.size, 0);
  s.add(array);
  assert.sameValue(s.has(array), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([string, string]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(string), true);
  s.delete(string);
  assert.sameValue(s.size, 0);
  s.add(string);
  assert.sameValue(s.has(string), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([booleanTrue, booleanTrue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(booleanTrue), true);
  s.delete(booleanTrue);
  assert.sameValue(s.size, 0);
  s.add(booleanTrue);
  assert.sameValue(s.has(booleanTrue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([booleanFalse, booleanFalse]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(booleanFalse), true);
  s.delete(booleanFalse);
  assert.sameValue(s.size, 0);
  s.add(booleanFalse);
  assert.sameValue(s.has(booleanFalse), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([functionExprValue, functionExprValue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(functionExprValue), true);
  s.delete(functionExprValue);
  assert.sameValue(s.size, 0);
  s.add(functionExprValue);  assert.sameValue(s.has(functionExprValue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([arrowFunctionValue, arrowFunctionValue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(arrowFunctionValue), true);
  s.delete(arrowFunctionValue);
  assert.sameValue(s.size, 0);
  s.add(arrowFunctionValue);  assert.sameValue(s.has(arrowFunctionValue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([classValue, classValue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(classValue), true);
  s.delete(classValue);
  assert.sameValue(s.size, 0);
  s.add(classValue);
  assert.sameValue(s.has(classValue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([map, map]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(map), true);
  s.delete(map);
  assert.sameValue(s.size, 0);
  s.add(map);
  assert.sameValue(s.has(map), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([set, set]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(set), true);
  s.delete(set);
  assert.sameValue(s.size, 0);
  s.add(set);
  assert.sameValue(s.has(set), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([weakMap, weakMap]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(weakMap), true);
  s.delete(weakMap);
  assert.sameValue(s.size, 0);
  s.add(weakMap);
  assert.sameValue(s.has(weakMap), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([weakRef, weakRef]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(weakRef), true);
  s.delete(weakRef);
  assert.sameValue(s.size, 0);
  s.add(weakRef);
  assert.sameValue(s.has(weakRef), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([weakSet, weakSet]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(weakSet), true);
  s.delete(weakSet);
  assert.sameValue(s.size, 0);
  s.add(weakSet);
  assert.sameValue(s.has(weakSet), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([nullValue, nullValue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(nullValue), true);
  s.delete(nullValue);
  assert.sameValue(s.size, 0);
  s.add(nullValue);
  assert.sameValue(s.has(nullValue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([undefinedValue, undefinedValue]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(undefinedValue), true);
  s.delete(undefinedValue);
  assert.sameValue(s.size, 0);
  s.add(undefinedValue);
  assert.sameValue(s.has(undefinedValue), true);
  assert.sameValue(s.size, 1);
};

{
  const s = new Set([unassigned, unassigned]);
  assert.sameValue(s.size, 1);
  assert.sameValue(s.has(unassigned), true);
  s.delete(unassigned);
  assert.sameValue(s.size, 0);
  s.add(unassigned);
  assert.sameValue(s.has(unassigned), true);
  assert.sameValue(s.size, 1);
};

