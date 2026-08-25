// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.name
description: >
  The initial value of Error.prototype.name is "Error".
---*/

assert.sameValue(Error.prototype.name, 'Error');

