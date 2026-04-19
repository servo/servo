// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 * Author: Tom Schuster
 */

JSON.stringify(new Boolean(false), function(k, v) { 
    assert.sameValue(typeof v, "object"); 
});

assert.sameValue(Boolean.prototype.hasOwnProperty('toJSON'), false);

Object.prototype.toJSON = function() { return 2; };
assert.sameValue(JSON.stringify(new Boolean(true)), "2");
