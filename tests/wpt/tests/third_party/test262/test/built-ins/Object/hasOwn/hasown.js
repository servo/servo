// Copyright 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
info: |
    Object.hasOwn ( _O_, _P_ )

    1. Let _obj_ be ? ToObject(_O_).
    2. Let _key_ be ? ToPropertyKey(_P_).
    3. Return ? HasOwnProperty(_obj_, _key_).
description: >
    Checking type of the Object.hasOwn and the returned result
author: Jamie Kyle
features: [Object.hasOwn]
---*/

assert.sameValue(typeof Object.hasOwn, 'function');
assert(Object.hasOwn(Object, 'hasOwn'));
