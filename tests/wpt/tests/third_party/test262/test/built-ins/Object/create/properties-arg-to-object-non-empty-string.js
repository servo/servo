// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.create
description: >
  Throws a TypeError if the Properties argument is a non-empty string
info: |
  Object.create ( O, Properties )

  3. If Properties is not undefined, then
    a. Return ? ObjectDefineProperties(obj, Properties).

  Runtime Semantics: ObjectDefineProperties ( O, Properties )

  2. Let props be ? ToObject(Properties).
  3. Let keys be ? props.[[OwnPropertyKeys]]().
  4. Let descriptors be a new empty List.
  5. For each element nextKey of keys in List order, do
    a. Let propDesc be ? props.[[GetOwnProperty]](nextKey).
    b. If propDesc is not undefined and propDesc.[[Enumerable]] is true, then
      i. Let descObj be ? Get(props, nextKey).
      ii. Let desc be ? ToPropertyDescriptor(descObj).

  ToPropertyDescriptor ( Obj )

  1. If Type(Obj) is not Object, throw a TypeError exception.
---*/

// The first nextKey is 'h' and its OwnProperty in the String object is enumerable
// Get(props, nextKey) is an equivalent of Object('hello')[nextKey]
// The first descObj will be 'h', so it will throw in ToPropertyDescriptor
assert.throws(TypeError, function() {
  Object.create({}, 'hello');
});
