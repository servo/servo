// Copyright (c) 2014 Hank Yates. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.from
description: Testing Array.from when passed a String
author: Hank Yates (hankyates@gmail.com)
---*/

var arrLikeSource = 'Test';
var result = Array.from(arrLikeSource);

assert.sameValue(result.length, 4, 'The value of result.length is expected to be 4');
assert.sameValue(result[0], 'T', 'The value of result[0] is expected to be "T"');
assert.sameValue(result[1], 'e', 'The value of result[1] is expected to be "e"');
assert.sameValue(result[2], 's', 'The value of result[2] is expected to be "s"');
assert.sameValue(result[3], 't', 'The value of result[3] is expected to be "t"');
