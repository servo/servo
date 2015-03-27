function sameDate(d1, d2) {
  return (d1 instanceof Date && d2 instanceof Date && d1.valueOf() == d2.valueOf());
}

function sameRE(r1, r2) {
  return (r1 instanceof RegExp && r2 instanceof RegExp && r1.toString() == r2.toString());
}

function assert_array_equals_(observed, expected, msg) {
  if (observed.length == expected.length) {
    for (var i = 0; i < observed.length; i++) {
      if (observed[i] instanceof Date) {
        observed[i] = sameDate(observed[i], expected[i]);
        expected[i] = true;
      } else if (observed[i] instanceof RegExp) {
        observed[i] = sameRE(observed[i], expected[i]);
        expected[i] = true;
      }
    }
  }

  assert_array_equals(observed, expected, msg);
}

function assert_object_equals_(observed, expected, msg) {
  for (var p in observed) {
    if (observed[p] instanceof Date) {
      observed[p] = sameDate(observed[p], expected[p]);
      expected[p] = true;
    } else if (observed[p] instanceof RegExp) {
      observed[p] = sameRE(observed[p], expected[p]);
      expected[p] = true;
    } else if (observed[p] instanceof Array || String(observed[p]) === '[object Object]') {
      observed[p] = String(observed[p]) === String(expected[p]);
      expected[p] = true;
    }
    assert_equals(observed[p], expected[p], msg);
  }
}
