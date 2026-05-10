/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
for(let c of [0, 0, 0, 0, 0]) {
  try { (function() {
      this.c = this;
      this.e = arguments;
      Object.defineProperty(this, "d", {
        get: Math.pow,
        configurable: true
      });
      delete this.e;
      delete this.c;
      Object.defineProperty(this, "d", {
        writable: true
      });
      if (Math.tan( - 1)) {
        Object.defineProperty(this, {});
      }
    } (c));
  } catch(e) {}
}

