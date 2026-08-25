// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Sets an ordinary property value if numeric key is not a CanonicalNumericIndex
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
    ...
  4. Return OrdinaryDefineOwnProperty(O, P, Desc).
  ...
includes: [testTypedArray.js, propertyHelper.js]
features: [BigInt, Reflect, TypedArray]
---*/

var keys = [
  "1.0",
  "+1",
  "1000000000000000000000",
  "0.0000001"
];

var fnset = function() {};
var fnget = function() {};

var acDesc = {
  get: fnget,
  set: fnset,
  enumerable: true,
  configurable: false
};

testWithBigIntTypedArrayConstructors(function(TA) {
  keys.forEach(function(key) {
    var dataDesc = {
      value: 42n,
      writable: true,
      configurable: true
    };

    var sample1 = new TA();

    assert.sameValue(
      Reflect.defineProperty(sample1, key, dataDesc),
      true,
      "return true after defining data property [" + key + "]"
    );

    assert.sameValue(sample1[key], 42n, "value is set to [" + key + "]");
    verifyNotEnumerable(sample1, key);
    verifyWritable(sample1, key);
    verifyConfigurable(sample1, key);

    assert.sameValue(sample1[0], undefined, "no value is set on sample1[0]");
    assert.sameValue(sample1.length, 0, "length is still 0");

    var sample2 = new TA();

    assert.sameValue(
      Reflect.defineProperty(sample2, key, acDesc),
      true,
      "return true after defining accessors property [" + key + "]"
    );

    var desc = Object.getOwnPropertyDescriptor(sample2, key);
    verifyEnumerable(sample2, key);
    assert.sameValue(desc.get, fnget, "accessor's get [" + key + "]");
    assert.sameValue(desc.set, fnset, "accessor's set [" + key + "]");
    verifyNotConfigurable(sample2, key);

    assert.sameValue(sample2[0], undefined,"no value is set on sample2[0]");
    assert.sameValue(sample2.length, 0, "length is still 0");

    var sample3 = new TA();
    Object.preventExtensions(sample3);

    assert.sameValue(
      Reflect.defineProperty(sample3, key, dataDesc),
      false,
      "return false defining property on a non-extensible sample"
    );
    assert.sameValue(Object.getOwnPropertyDescriptor(sample3, key), undefined);

    var sample4 = new TA();
    Object.preventExtensions(sample4);

    assert.sameValue(
      Reflect.defineProperty(sample4, key, acDesc),
      false,
      "return false defining property on a non-extensible sample"
    );
    assert.sameValue(Object.getOwnPropertyDescriptor(sample4, key), undefined);
  });
}, null, ["passthrough"]);
