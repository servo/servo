// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.3.1
description: >
    Tests that Intl.Collator.prototype.constructor is the
    Intl.Collator.
---*/

assert.sameValue(Intl.Collator.prototype.constructor, Intl.Collator, "Intl.Collator.prototype.constructor is not the same as Intl.Collator");
