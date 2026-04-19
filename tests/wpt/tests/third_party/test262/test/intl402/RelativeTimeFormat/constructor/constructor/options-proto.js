// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: |
    Checks that the RelativeTimeFormat constructor does not cause the
    NumberFormat and PluralRules constructors to get properties off
    Object.prototype through the options objects it creates.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    20. Let nfOptions be ObjectCreate(null).
    25. Let prOptions be ObjectCreate(null).
features: [Intl.RelativeTimeFormat]
---*/

Object.defineProperties(Object.prototype, {
  // NumberFormat & PluralRules
  "localeMatcher": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: localeMatcher");
    },
  },

  "minimumIntegerDigits": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: minimumIntegerDigits");
    },
  },

  "minimumFractionDigits": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: minimumFractionDigits");
    },
  },

  "maximumFractionDigits": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: maximumFractionDigits");
    },
  },

  "minimumSignificantDigits": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: minimumSignificantDigits");
    },
  },

  "maximumSignificantDigits": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: maximumSignificantDigits");
    },
  },

  // NumberFormat
  "style": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: style");
    },
  },

  "currency": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: currency");
    },
  },

  "currencyDisplay": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: currencyDisplay");
    },
  },

  "useGrouping": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: useGrouping");
    },
  },

  // PluralRules
  "type": {
    "get": function() {
      throw new Test262Error("Should not call getter on Object.prototype: type");
    },
  },
});

new Intl.RelativeTimeFormat();
