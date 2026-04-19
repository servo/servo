// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.message
description: The initial value of Error.prototype.message is the empty String.
---*/

assert.sameValue(Error('a').message, "a", 'The value of err1.message is "a"');
assert.sameValue(new Error('a').message, "a", 'The value of err1.message is "a"');
assert(!Error().hasOwnProperty('message'));
assert(!new Error().hasOwnProperty('message'));
assert.sameValue(new Error().message, Error.prototype.message, 'The value of new Error().message equals Error.prototype.message');
