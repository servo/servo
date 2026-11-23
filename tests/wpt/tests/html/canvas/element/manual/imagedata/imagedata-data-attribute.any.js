// META: title=ImageData's data attribute is an accessor on ImageData.prototype

test(() => {
  const imageData = new ImageData(1, 1);
  assert_array_equals(Object.getOwnPropertyNames(imageData), []);
  assert_false(imageData.hasOwnProperty("data"));
}, "ImageData instances have no own properties");

test(() => {
  const descriptor = Object.getOwnPropertyDescriptor(ImageData.prototype, "data");
  assert_not_equals(descriptor, undefined);
  assert_equals(typeof descriptor.get, "function");
  assert_equals(descriptor.set, undefined);
  assert_true(descriptor.enumerable);
  assert_true(descriptor.configurable);
}, "ImageData.prototype.data is a readonly accessor property");

test(() => {
  const imageData = new ImageData(1, 1);
  assert_true(imageData.data instanceof Uint8ClampedArray);
  assert_equals(imageData.data.length, 4);
  assert_equals(imageData.data, imageData.data);
}, "ImageData's data getter returns the same Uint8ClampedArray each time");
