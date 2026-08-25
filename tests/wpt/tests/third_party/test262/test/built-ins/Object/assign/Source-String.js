// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Test Object.Assign(target,...sources), string have own enumerable properties, so it can be wrapped to objects.
esid: sec-object.assign
---*/

var target = new Object();
var result = Object.assign(target, "123");

assert.sameValue(result[0], "1", 'The value of result[0] is expected to be "1"');
assert.sameValue(result[1], "2", 'The value of result[1] is expected to be "2"');
assert.sameValue(result[2], "3", 'The value of result[2] is expected to be "3"');
