// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl-pluralrules-abstracts
description: >
    Tests that the behavior of a Record is not affected by
    adversarial  changes to Object.prototype.
info: |
  1.1.1. InitializePluralRules (pluralRules, locales, options)
  (...)
  1.1.1_6. Let t be ? GetOption(options, "type", "string", « "cardinal", "ordinal" », "cardinal").
author: Zibi Braniecki
includes: [testIntl.js]
---*/

taintProperties(["type"]);

var pr = new Intl.PluralRules();
var time = pr.select(9);
