// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-properties-of-intl-pluralrules-prototype-object
description: >
    Tests that Intl.PluralRules.prototype meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Zibi Braniecki
---*/

assert(Object.isExtensible(Intl.PluralRules.prototype), "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.PluralRules.prototype), Object.prototype,
                 "Built-in prototype objects must have Object.prototype as their prototype.");
