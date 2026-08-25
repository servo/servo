// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-intl-collator-prototype-object
description: >
    Tests that Intl.Collator.prototype is not an object that has been
    initialized as an Intl.Collator.
---*/

// test by calling a function that should fail as "this" is not an object
// initialized as an Intl.Collator
assert.throws(TypeError, () => Intl.Collator.prototype.compare("aаあ아", "aаあ아"),
              "Intl.Collator.prototype is not an object that has been initialized as an Intl.Collator.");
