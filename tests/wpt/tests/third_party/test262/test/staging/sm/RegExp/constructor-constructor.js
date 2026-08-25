// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp constructor should check pattern.constructor.
info: bugzilla.mozilla.org/show_bug.cgi?id=1147817
esid: pending
---*/

var g = $262.createRealm().global;

var re = /foo/;
assert.sameValue(RegExp(re), re);
re.constructor = 10;
assert.sameValue(RegExp(re) === re, false);
assert.sameValue(RegExp(re).toString(), re.toString());

// If pattern comes from different global, RegExp shouldn't return it.
re = g.eval(`var re = /foo/; re;`);
assert.sameValue(RegExp(re) === re, false);
assert.sameValue(RegExp(re).toString(), re.toString());
g.eval(`re.constructor = 10;`);
assert.sameValue(RegExp(re) === re, false);
assert.sameValue(RegExp(re).toString(), re.toString());


re = new Proxy(/a/, {
  get(that, name) {
    return that[name];
  }
});
assert.sameValue(RegExp(re), re);
re = new Proxy(/a/, {
  get(that, name) {
    if (name == "constructor") {
      return function() {};
    }
    return that[name];
  }
});
assert.sameValue(RegExp(re) === re, false);
re = new Proxy(/a/, {
  get(that, name) {
    if (name == Symbol.match) {
      return undefined;
    }
    return that[name];
  }
});
assert.sameValue(RegExp(re) === re, false);

re = new Proxy(g.eval(`/a/`), {
  get(that, name) {
    return that[name];
  }
});
assert.sameValue(RegExp(re) === re, false);

re = g.eval(`new Proxy(/a/, {
  get(that, name) {
    return that[name];
  }
});`);
assert.sameValue(RegExp(re) === re, false);


var obj = { [Symbol.match]: true, source: "foo", flags: "gi" };
assert.sameValue(RegExp(obj) === obj, false);
assert.sameValue(RegExp(obj).toString(), "/foo/gi");
obj.constructor = RegExp;
assert.sameValue(RegExp(obj), obj);

obj = g.eval(`var obj = { [Symbol.match]: true, source: "foo", flags: "gi" }; obj;`);
assert.sameValue(RegExp(obj) === obj, false);
assert.sameValue(RegExp(obj).toString(), "/foo/gi");
g.eval(`obj.constructor = RegExp`);
assert.sameValue(RegExp(obj) === obj, false);
obj.constructor = RegExp;
assert.sameValue(RegExp(obj), obj);
