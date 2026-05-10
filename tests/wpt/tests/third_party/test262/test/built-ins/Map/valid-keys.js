// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.set
description: Observing the expected behavior of valid keys
info: |
  Map.prototype.set ( key , value )

  ...
  Let p be the Record {[[key]]: key, [[value]]: value}.
  Append p as the last element of entries.
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
  const m = new Map([[negativeZero, negativeZero]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(negativeZero), true);
  assert.sameValue(m.get(negativeZero), negativeZero);
  m.delete(negativeZero);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(negativeZero), false);
  m.set(negativeZero, negativeZero);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(negativeZero), true);
  assert.sameValue(m.get(negativeZero), negativeZero);
};

{
  const m = new Map([[positiveZero, positiveZero]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(positiveZero), true);
  assert.sameValue(m.get(positiveZero), positiveZero);
  m.delete(positiveZero);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(positiveZero), false);
  m.set(positiveZero, positiveZero);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(positiveZero), true);
  assert.sameValue(m.get(positiveZero), positiveZero);
};

{
  const m = new Map([[zero, zero]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(zero), true);
  assert.sameValue(m.get(zero), zero);
  m.delete(zero);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(zero), false);
  m.set(zero, zero);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(zero), true);
  assert.sameValue(m.get(zero), zero);
};

{
  const m = new Map([[one, one]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(one), true);
  assert.sameValue(m.get(one), one);
  m.delete(one);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(one), false);
  m.set(one, one);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(one), true);
  assert.sameValue(m.get(one), one);
};

{
  const m = new Map([[twoRaisedToFiftyThreeMinusOne, twoRaisedToFiftyThreeMinusOne]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(twoRaisedToFiftyThreeMinusOne), true);
  assert.sameValue(m.get(twoRaisedToFiftyThreeMinusOne), twoRaisedToFiftyThreeMinusOne);
  m.delete(twoRaisedToFiftyThreeMinusOne);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(twoRaisedToFiftyThreeMinusOne), false);
  m.set(twoRaisedToFiftyThreeMinusOne, twoRaisedToFiftyThreeMinusOne);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(twoRaisedToFiftyThreeMinusOne), true);
  assert.sameValue(m.get(twoRaisedToFiftyThreeMinusOne), twoRaisedToFiftyThreeMinusOne);
};

{
  const m = new Map([[int32Array, int32Array]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(int32Array), true);
  assert.sameValue(m.get(int32Array), int32Array);
  m.delete(int32Array);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(int32Array), false);
  m.set(int32Array, int32Array);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(int32Array), true);
  assert.sameValue(m.get(int32Array), int32Array);
};

{
  const m = new Map([[uint32Array, uint32Array]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(uint32Array), true);
  assert.sameValue(m.get(uint32Array), uint32Array);
  m.delete(uint32Array);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(uint32Array), false);
  m.set(uint32Array, uint32Array);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(uint32Array), true);
  assert.sameValue(m.get(uint32Array), uint32Array);
};

{
  const m = new Map([[n, n]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n), true);
  assert.sameValue(m.get(n), n);
  m.delete(n);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(n), false);
  m.set(n, n);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n), true);
  assert.sameValue(m.get(n), n);
};

{
  const m = new Map([[bigInt, bigInt]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigInt), true);
  assert.sameValue(m.get(bigInt), bigInt);
  m.delete(bigInt);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(bigInt), false);
  m.set(bigInt, bigInt);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigInt), true);
  assert.sameValue(m.get(bigInt), bigInt);
};

{
  const m = new Map([[n1, n1]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n1), true);
  assert.sameValue(m.get(n1), n1);
  m.delete(n1);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(n1), false);
  m.set(n1, n1);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n1), true);
  assert.sameValue(m.get(n1), n1);
};

{
  const m = new Map([[n53, n53]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n53), true);
  assert.sameValue(m.get(n53), n53);
  m.delete(n53);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(n53), false);
  m.set(n53, n53);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(n53), true);
  assert.sameValue(m.get(n53), n53);
};

{
  const m = new Map([[fiftyThree, fiftyThree]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(fiftyThree), true);
  assert.sameValue(m.get(fiftyThree), fiftyThree);
  m.delete(fiftyThree);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(fiftyThree), false);
  m.set(fiftyThree, fiftyThree);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(fiftyThree), true);
  assert.sameValue(m.get(fiftyThree), fiftyThree);
};

{
  const m = new Map([[bigInt64Array, bigInt64Array]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigInt64Array), true);
  assert.sameValue(m.get(bigInt64Array), bigInt64Array);
  m.delete(bigInt64Array);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(bigInt64Array), false);
  m.set(bigInt64Array, bigInt64Array);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigInt64Array), true);
  assert.sameValue(m.get(bigInt64Array), bigInt64Array);
};

{
  const m = new Map([[bigUint64Array, bigUint64Array]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigUint64Array), true);
  assert.sameValue(m.get(bigUint64Array), bigUint64Array);
  m.delete(bigUint64Array);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(bigUint64Array), false);
  m.set(bigUint64Array, bigUint64Array);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(bigUint64Array), true);
  assert.sameValue(m.get(bigUint64Array), bigUint64Array);
};

{
  const m = new Map([[symbol, symbol]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(symbol), true);
  assert.sameValue(m.get(symbol), symbol);
  m.delete(symbol);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(symbol), false);
  m.set(symbol, symbol);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(symbol), true);
  assert.sameValue(m.get(symbol), symbol);
};

{
  const m = new Map([[object, object]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(object), true);
  assert.sameValue(m.get(object), object);
  m.delete(object);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(object), false);
  m.set(object, object);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(object), true);
  assert.sameValue(m.get(object), object);
};

{
  const m = new Map([[array, array]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(array), true);
  assert.sameValue(m.get(array), array);
  m.delete(array);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(array), false);
  m.set(array, array);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(array), true);
  assert.sameValue(m.get(array), array);
};

{
  const m = new Map([[string, string]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(string), true);
  assert.sameValue(m.get(string), string);
  m.delete(string);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(string), false);
  m.set(string, string);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(string), true);
  assert.sameValue(m.get(string), string);
};

{
  const m = new Map([[booleanTrue, booleanTrue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(booleanTrue), true);
  assert.sameValue(m.get(booleanTrue), booleanTrue);
  m.delete(booleanTrue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(booleanTrue), false);
  m.set(booleanTrue, booleanTrue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(booleanTrue), true);
  assert.sameValue(m.get(booleanTrue), booleanTrue);
};

{
  const m = new Map([[booleanFalse, booleanFalse]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(booleanFalse), true);
  assert.sameValue(m.get(booleanFalse), booleanFalse);
  m.delete(booleanFalse);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(booleanFalse), false);
  m.set(booleanFalse, booleanFalse);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(booleanFalse), true);
  assert.sameValue(m.get(booleanFalse), booleanFalse);
};

{
  const m = new Map([[functionExprValue, functionExprValue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(functionExprValue), true);
  assert.sameValue(m.get(functionExprValue), functionExprValue);
  m.delete(functionExprValue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(functionExprValue), false);
  m.set(functionExprValue, functionExprValue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(functionExprValue), true);
  assert.sameValue(m.get(functionExprValue), functionExprValue);
};

{
  const m = new Map([[arrowFunctionValue, arrowFunctionValue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(arrowFunctionValue), true);
  assert.sameValue(m.get(arrowFunctionValue), arrowFunctionValue);
  m.delete(arrowFunctionValue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(arrowFunctionValue), false);
  m.set(arrowFunctionValue, arrowFunctionValue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(arrowFunctionValue), true);
  assert.sameValue(m.get(arrowFunctionValue), arrowFunctionValue);
};

{
  const m = new Map([[classValue, classValue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(classValue), true);
  assert.sameValue(m.get(classValue), classValue);
  m.delete(classValue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(classValue), false);
  m.set(classValue, classValue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(classValue), true);
  assert.sameValue(m.get(classValue), classValue);
};

{
  const m = new Map([[map, map]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(map), true);
  assert.sameValue(m.get(map), map);
  m.delete(map);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(map), false);
  m.set(map, map);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(map), true);
  assert.sameValue(m.get(map), map);
};

{
  const m = new Map([[set, set]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(set), true);
  assert.sameValue(m.get(set), set);
  m.delete(set);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(set), false);
  m.set(set, set);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(set), true);
  assert.sameValue(m.get(set), set);
};

{
  const m = new Map([[weakMap, weakMap]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakMap), true);
  assert.sameValue(m.get(weakMap), weakMap);
  m.delete(weakMap);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(weakMap), false);
  m.set(weakMap, weakMap);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakMap), true);
  assert.sameValue(m.get(weakMap), weakMap);
};

{
  const m = new Map([[weakRef, weakRef]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakRef), true);
  assert.sameValue(m.get(weakRef), weakRef);
  m.delete(weakRef);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(weakRef), false);
  m.set(weakRef, weakRef);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakRef), true);
  assert.sameValue(m.get(weakRef), weakRef);
};

{
  const m = new Map([[weakSet, weakSet]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakSet), true);
  assert.sameValue(m.get(weakSet), weakSet);
  m.delete(weakSet);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(weakSet), false);
  m.set(weakSet, weakSet);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(weakSet), true);
  assert.sameValue(m.get(weakSet), weakSet);
};

{
  const m = new Map([[nullValue, nullValue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(nullValue), true);
  assert.sameValue(m.get(nullValue), nullValue);
  m.delete(nullValue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(nullValue), false);
  m.set(nullValue, nullValue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(nullValue), true);
  assert.sameValue(m.get(nullValue), nullValue);
};

{
  const m = new Map([[undefinedValue, undefinedValue]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(undefinedValue), true);
  assert.sameValue(m.get(undefinedValue), undefinedValue);
  m.delete(undefinedValue);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(undefinedValue), false);
  m.set(undefinedValue, undefinedValue);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(undefinedValue), true);
  assert.sameValue(m.get(undefinedValue), undefinedValue);
};

{
  const m = new Map([[unassigned, unassigned]]);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(unassigned), true);
  assert.sameValue(m.get(unassigned), unassigned);
  m.delete(unassigned);
  assert.sameValue(m.size, 0);
  assert.sameValue(m.has(unassigned), false);
  m.set(unassigned, unassigned);
  assert.sameValue(m.size, 1);
  assert.sameValue(m.has(unassigned), true);
  assert.sameValue(m.get(unassigned), unassigned);
};

