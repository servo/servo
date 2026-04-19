// Copyright (C) 2024 Jordan Harband.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: >
  Returns false on symbols
features: [Error.isError, Symbol]
---*/

assert.sameValue(Error.isError(Symbol()), false);
