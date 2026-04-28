// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks the order of operations on the options argument to the ListFormat constructor.
info: |
    Intl.ListFormat ( [ locales [ , options ] ] )
    7. Let type be GetOption(options, "type", "string", « "conjunction", "disjunction", "unit" », "conjunction").
    9. Let style be GetOption(options, "style", "string", « "long", "short", "narrow" », "long").
    12. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
includes: [compareArray.js]
features: [Intl.ListFormat]
---*/

const callOrder = [];

new Intl.ListFormat([], {
  get localeMatcher() {
    callOrder.push("localeMatcher");
    return {
      toString() {
        callOrder.push("localeMatcher toString");
        return "best fit";
      }
    };
  },

  get type() {
    callOrder.push("type");
    return {
      toString() {
        callOrder.push("type toString");
        return "unit";
      }
    };
  },

  get style() {
    callOrder.push("style");
    return {
      toString() {
        callOrder.push("style toString");
        return "short";
      }
    };
  },
});

assert.compareArray(callOrder, [
  "localeMatcher",
  "localeMatcher toString",
  "type",
  "type toString",
  "style",
  "style toString",
]);
