/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Better/more correct handling for replacer arrays with getter array index properties
info: bugzilla.mozilla.org/show_bug.cgi?id=648471
esid: pending
---*/

/* JSID_INT_MIN/MAX copied from jsapi.h. */

var obj =
  {
    /* [JSID_INT_MIN - 1, JSID_INT_MIN + 1] */
    "-1073741825": -1073741825,
    "-1073741824": -1073741824,
    "-1073741823": -1073741823,

    "-2.5": -2.5,
    "-1": -1,

    0: 0,

    1: 1,
    2.5: 2.5,

    /* [JSID_INT_MAX - 1, JSID_INT_MAX + 1] */
    1073741822: 1073741822,
    1073741823: 1073741823,
    1073741824: 1073741824,
  };

for (var s in obj)
{
  var n = obj[s];
  assert.sameValue(+s, n);
  assert.sameValue(JSON.stringify(obj, [n]),
           '{"' + s + '":' + n + '}',
           "Failed to stringify numeric property " + n + "correctly");
  assert.sameValue(JSON.stringify(obj, [s]),
           '{"' + s + '":' + n + '}',
           "Failed to stringify string property " + n + "correctly");
  assert.sameValue(JSON.stringify(obj, [s, ]),
           '{"' + s + '":' + n + '}',
           "Failed to stringify string then number properties ('" + s + "', " + n + ") correctly");
  assert.sameValue(JSON.stringify(obj, [n, s]),
           '{"' + s + '":' + n + '}',
           "Failed to stringify number then string properties (" + n + ", '" + s + "') correctly");
}

// -0 is tricky, because ToString(-0) === "0", so test it specially.
assert.sameValue(JSON.stringify({ "-0": 17, 0: 42 }, [-0]),
         '{"0":42}',
         "Failed to stringify numeric property -0 correctly");
assert.sameValue(JSON.stringify({ "-0": 17, 0: 42 }, ["-0"]),
         '{"-0":17}',
         "Failed to stringify string property -0 correctly");
assert.sameValue(JSON.stringify({ "-0": 17, 0: 42 }, ["-0", -0]),
         '{"-0":17,"0":42}',
         "Failed to stringify string then number properties ('-0', -0) correctly");
assert.sameValue(JSON.stringify({ "-0": 17, 0: 42 }, [-0, "-0"]),
         '{"0":42,"-0":17}',
         "Failed to stringify number then string properties (-0, '-0) correctly");
