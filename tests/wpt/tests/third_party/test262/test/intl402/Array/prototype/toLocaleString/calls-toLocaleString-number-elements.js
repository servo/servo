// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-array.prototype.tolocalestring
description: >
  Ensure "toLocaleString" is called with locale and options on number elements.
---*/

var n = 0;

var locale = "th-u-nu-thai";
var options = {
    minimumFractionDigits: 3
};

assert.sameValue([n].toLocaleString(locale, options), n.toLocaleString(locale, options));
