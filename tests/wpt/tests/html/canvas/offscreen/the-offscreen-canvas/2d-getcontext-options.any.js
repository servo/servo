test(() => {
  const expected = [
    "alpha",
    "colorSpace",
    "colorSpace toString",
    "desynchronized",
    "willReadFrequently",
  ];
  var actual = [];
  const options = {
    get alpha() {
      actual.push("alpha");
      return true;
    },
    get willReadFrequently() {
      actual.push("willReadFrequently");
      return false;
    },
    get desynchronized() {
      actual.push("desynchronized");
      return false;
    },
    get colorSpace() {
      actual.push("colorSpace");
      return {
        toString() {
          actual.push("colorSpace toString");
          return "srgb";
        }
      };
    },
  };

  const canvas = new OffscreenCanvas(100, 50);
  const context = canvas.getContext('2d', options);
  assert_not_equals(context, null, "context");
  assert_array_equals(actual, expected, "order of operations (creation)");
  actual = [];
  assert_equals(canvas.getContext('2d', options), context, "cached context");
  assert_array_equals(actual, expected, "order of operations (caching)");
});
