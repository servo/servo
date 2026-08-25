// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks the order of operations on the options argument to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    7. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
    11. Let granularity be ? GetOption(options, "granularity", "string", « "grapheme", "word", "sentence" », "grapheme").
includes: [compareArray.js]
features: [Intl.Segmenter]
---*/

const callOrder = [];

new Intl.Segmenter([], {
  get localeMatcher() {
    callOrder.push("localeMatcher");
    return {
      toString() {
        callOrder.push("localeMatcher toString");
        return "best fit";
      }
    };
  },
  get lineBreakStyle() {
    callOrder.push("lineBreakStyle");
    return {
      toString() {
        callOrder.push("lineBreakStyle toString");
        return "strict";
      }
    };
  },
  get granularity() {
    callOrder.push("granularity");
    return {
      toString() {
        callOrder.push("granularity toString");
        return "word";
      }
    };
  },
});

assert.compareArray(callOrder, [
  "localeMatcher",
  "localeMatcher toString",
  "granularity",
  "granularity toString",
]);
