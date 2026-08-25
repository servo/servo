// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-intl-datetimeformat-prototype-object
description: >
    Tests that Intl.DateTimeFormat.prototype is not an object that has
    been initialized as an Intl.DateTimeFormat.
author: Roozbeh Pournader
---*/

// test by calling a function that should fail as "this" is not an object
// initialized as an Intl.DateTimeFormat
assert.throws(TypeError, () => Intl.DateTimeFormat.prototype.format(0),
              "Intl.DateTimeFormat's prototype is not an object that has been initialized as an Intl.DateTimeFormat");
