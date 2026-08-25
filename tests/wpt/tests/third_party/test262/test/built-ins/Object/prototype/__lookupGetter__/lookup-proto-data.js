// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-additional-properties-of-the-object.prototype-object
description: Behavior when prototype defines a like-named data property
info: |
    [...]
    4. Repeat
       a. Let desc be ? O.[[GetOwnProperty]](key).
       b. If desc is not undefined, then
          i. If IsAccessorDescriptor(desc) is true, return desc.[[Get]].
          ii. Return undefined.
       c. Let O be ? O.[[GetPrototypeOf]]().
       d. If O is null, return undefined.
features: [__getter__]
---*/

var root = Object.defineProperty({}, 'target', { get: function() {} });
var desc = { value: null };
var intermediary = Object.create(root, { target: desc });
var subject = Object.create(intermediary);

assert.sameValue(subject.__lookupGetter__('target'), undefined);
