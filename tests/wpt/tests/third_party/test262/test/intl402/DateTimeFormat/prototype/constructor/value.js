// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.3.1
description: >
    Tests that Intl.DateTimeFormat.prototype.constructor is the
    Intl.DateTimeFormat.
author: Roozbeh Pournader
---*/

assert.sameValue(Intl.DateTimeFormat.prototype.constructor, Intl.DateTimeFormat, "Intl.DateTimeFormat.prototype.constructor is not the same as Intl.DateTimeFormat");
