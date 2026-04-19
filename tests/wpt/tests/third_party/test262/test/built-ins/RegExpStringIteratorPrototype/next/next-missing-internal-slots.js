// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Throws TypeError when `this` does not have all internal slots
info: |
  %RegExpStringIteratorPrototype%.next ( )
    1. Let O be the this value.
    2. If Type(O) is not Object, throw a TypeError exception.
    3. If O does not have all of the internal slots of a RegExp String Iterator
       Object Instance (see PropertiesOfRegExpStringIteratorInstances), throw a
       TypeError.
features: [Symbol.matchAll]
---*/

var iterator = /./[Symbol.matchAll]('');
var object = Object.create(iterator);

assert.throws(TypeError, function() {
  object.next();
});
