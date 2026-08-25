// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  Object.assign with an Array Exotic Object target employs the corresponding
  internal methods.
info: |
  Object.assign ( _target_, ..._sources_ )
  3.a.iii.2.b. Perform ? Set(_to_, _nextKey_, _propValue_, *true*).

  Set ( _O_, _P_, _V_, _Throw_ )
  1. Let _success_ be ? _O_.[[Set]](_P_, _V_, _O_).

  OrdinarySet ( _O_, _P_, _V_, _Receiver_ )
  1. Let _ownDesc_ be ? _O_.[[GetOwnProperty]](_P_).
  2. Return ? OrdinarySetWithOwnDescriptor(_O_, _P_, _V_, _Receiver_, _ownDesc_).

  OrdinarySetWithOwnDescriptor ( _O_, _P_, _V_, _Receiver_, _ownDesc_ )
  1. If _ownDesc_ is *undefined*, then
     a. Let _parent_ be ? O.[[GetPrototypeOf]]().
     b. If _parent_ is not *null*, then
        i. Return ? _parent_.[[Set]](_P_, _V_, _Receiver_).
     c. Else,
        i. Set _ownDesc_ to the PropertyDescriptor { [[Value]]: *undefined*, [[Writable]]: *true*, [[Enumerable]]: *true*, [[Configurable]]: *true* }.
  2. If IsDataDescriptor(_ownDesc_) is *true*, then
     ...
     c. Let _existingDescriptor_ be ? _Receiver_.[[GetOwnProperty]](_P_).
     d. If _existingDescriptor_ is not *undefined*, then
        ...
        iii. Let _valueDesc_ be the PropertyDescriptor { [[Value]]: _V_ }.
        iv. Return ? _Receiver_.[[DefineOwnProperty]](_P_, _valueDesc_).
     e. Else,
        i. Assert: _Receiver_ does not currently have a property _P_.
        ii. Return ? CreateDataProperty(_Receiver_, _P_, _V_).

  CreateDataProperty ( _O_, _P_, _V_ )
  1. Let _newDesc_ be the PropertyDescriptor { [[Value]]: _V_, [[Writable]]: *true*, [[Enumerable]]: *true*, [[Configurable]]: *true* }.
  2. Return ? _O_.[[DefineOwnProperty]](_P_, _newDesc_).

  Array exotic object [[DefineOwnProperty]] ( _P_, _Desc_ )
  1. If _P_ is *"length"*, then
     a. Return ? ArraySetLength(_A_, _Desc_).
  2. Else if _P_ is an array index, then
     ...
     k. If _index_ ≥ _length_, then
        i. Set _lengthDesc_.[[Value]] to _index_ + *1*𝔽.
        ii. Set _succeeded_ to ! OrdinaryDefineOwnProperty(_A_, *"length"*, _lengthDesc_).
  3. Return ? OrdinaryDefineOwnProperty(_A_, _P_, _Desc_).

  The Object Type
  An **integer index** is a property name _n_ such that CanonicalNumericIndexString(_n_) returns an
  integral Number in the inclusive interval from *+0*𝔽 to 𝔽(2**53 - 1). An **array index** is an
  integer index _n_ such that CanonicalNumericIndexString(_n_) returns an integral Number in the
  inclusive interval from *+0*𝔽 to 𝔽(2**32 - 2).
---*/

var target = [7, 8, 9];
var result = Object.assign(target, [1]);
assert.sameValue(result, target);
assert.compareArray(result, [1, 8, 9],
  "elements must be assigned from an array source onto an array target");

var sparseArraySource = [];
sparseArraySource[2] = 3;
result = Object.assign(target, sparseArraySource);
assert.sameValue(result, target);
assert.compareArray(result, [1, 8, 3], "holes in a sparse array source must not be copied");

var shortObjectSource = { 1: 2, length: 2 };
shortObjectSource["-0"] = -1;
shortObjectSource["1.5"] = -2;
shortObjectSource["4294967295"] = -3; // 2**32 - 1
result = Object.assign(target, shortObjectSource);
assert.sameValue(result, target);
assert.compareArray(result, [1, 2],
  "array index properties must be copied from a non-array source");
assert.sameValue(result["-0"], -1,
  "a property with name -0 must be assigned onto an array target");
assert.sameValue(result["1.5"], -2,
  "a property with name 1.5 must be assigned onto an array target");
assert.sameValue(result["4294967295"], -3,
  "a property with name 4294967295 (2**32 - 1) must be assigned onto an array target");

result = Object.assign(target, { length: 1 });
assert.sameValue(result, target);
assert.compareArray(result, [1], "assigning a short length must shrink an array target");

result = Object.assign(target, { 2: 0 });
assert.sameValue(result, target);
assert.compareArray(result, [1, undefined, 0],
  "assigning a high array index must grow an array target");

if (typeof Proxy !== 'undefined') {
  var accordionSource = new Proxy({ length: 0, 1: 9 }, {
    ownKeys: function() {
      return ["length", "1"];
    }
  });
  result = Object.assign(target, accordionSource);
  assert.sameValue(result, target);
  assert.compareArray(result, [undefined, 9],
    "assigning a short length before a high array index must shrink and then grow an array target");
}
