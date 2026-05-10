// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.DurationFormat
description: >
    Tests that Intl.DurationFormat.prototype.constructor is the Intl.DurationFormat.

features: [Intl.DurationFormat]
---*/

assert.sameValue(Intl.DurationFormat.prototype.constructor, Intl.DurationFormat, "Intl.DurationFormat.prototype.constructor is not the same as Intl.DurationFormat");
