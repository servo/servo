// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns false on primitives
features: [Error.isError]
---*/

assert.sameValue(Error.isError(), false);
assert.sameValue(Error.isError(undefined), false);
assert.sameValue(Error.isError(null), false);
assert.sameValue(Error.isError(true), false);
assert.sameValue(Error.isError(false), false);
assert.sameValue(Error.isError(0), false);
assert.sameValue(Error.isError(-0), false);
assert.sameValue(Error.isError(NaN), false);
assert.sameValue(Error.isError(Infinity), false);
assert.sameValue(Error.isError(-Infinity), false);
assert.sameValue(Error.isError(42), false);
assert.sameValue(Error.isError(''), false);
assert.sameValue(Error.isError('foo'), false);
