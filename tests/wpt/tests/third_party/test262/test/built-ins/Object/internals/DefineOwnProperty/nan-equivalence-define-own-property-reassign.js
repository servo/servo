// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
  Replaces value field even if they pass in the SameValue algorithm, including
  distinct NaN values
info: |
  This test does not compare the actual byte values, instead it simply checks that
  the value is some valid NaN encoding.

  ---

  Previously, this test compared the "value" field using the SameValue
  algorithm (thereby ignoring distinct NaN values)

  ---

  [[DefineOwnProperty]] (P, Desc)

  Return ? OrdinaryDefineOwnProperty(O, P, Desc).

  #sec-ordinarydefineownproperty
  OrdinaryDefineOwnProperty ( O, P, Desc )

  1. Let current be ? O.[[GetOwnProperty]](P).
  2. Let extensible be O.[[Extensible]].
  3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc,
     current).

  #sec-validateandapplypropertydescriptor
  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )

  ...
  7. Else if IsDataDescriptor(current) and IsDataDescriptor(Desc) are both true,
     then
    a. If current.[[Configurable]] is false and current.[[Writable]] is false,
       then
      ...
  ...
  9. If O is not undefined, then
    a. For each field of Desc that is present, set the corresponding attribute
       of the property named P of object O to the value of the field.
  10. Return true.

  #sec-isnan-number

  NOTE: A reliable way for ECMAScript code to test if a value X is a NaN is
  an expression of the form  X !== X. The result will be true if and only
  if X is a NaN.
includes: [nans.js]
---*/

var len = NaNs.length;

for (var idx = 0; idx < len; ++idx) {
  for (var jdx = 0; jdx < len; ++jdx) {
    var a = {};

    a.prop = NaNs[idx];
    a.prop = NaNs[jdx];

    assert(
      a.prop !== a.prop,
      `Object property value reassigned to NaN produced by (index=${idx}) results in a valid NaN`
    );
  }
}
