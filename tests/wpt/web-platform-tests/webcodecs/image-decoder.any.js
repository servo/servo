// META: global=window,dedicatedworker
// META: script=/webcodecs/image-decoder-utils.js

function testFourColorsDecode(filename, mimeType) {
  return fetch(filename)
      .then(response => {
        let decoder = new ImageDecoder({data: response.body, type: mimeType});
        return decoder.decode();
      })
      .then(result => {
        assert_equals(result.image.displayWidth, 320);
        assert_equals(result.image.displayHeight, 240);

        let canvas = new OffscreenCanvas(
            result.image.displayWidth, result.image.displayHeight);
        let ctx = canvas.getContext('2d');
        ctx.drawImage(result.image, 0, 0);

        let top_left = toUInt32(ctx.getImageData(0, 0, 1, 1));
        assert_equals(top_left, 0xFFFF00FF, 'top left corner is yellow');

        let top_right =
            toUInt32(ctx.getImageData(result.image.displayWidth - 1, 0, 1, 1));
        assert_equals(top_right, 0xFF0000FF, 'top right corner is red');

        let bottom_left =
            toUInt32(ctx.getImageData(0, result.image.displayHeight - 1, 1, 1));
        assert_equals(bottom_left, 0x0000FFFF, 'bottom left corner is blue');

        let left_corner = toUInt32(ctx.getImageData(
            result.image.displayWidth - 1, result.image.displayHeight - 1, 1,
            1));
        assert_equals(left_corner, 0x00FF00FF, 'bottom right corner is green');
      });
}

promise_test(t => {
  return testFourColorsDecode('four-colors.jpg', 'image/jpeg');
}, 'Test JPEG image decoding.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(1);
}, 'Test JPEG w/ EXIF orientation top-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(2);
}, 'Test JPEG w/ EXIF orientation top-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(3);
}, 'Test JPEG w/ EXIF orientation bottom-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(4);
}, 'Test JPEG w/ EXIF orientation bottom-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(5);
}, 'Test JPEG w/ EXIF orientation left-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(6);
}, 'Test JPEG w/ EXIF orientation right-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(7);
}, 'Test JPEG w/ EXIF orientation right-bottom.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(8);
}, 'Test JPEG w/ EXIF orientation left-bottom.');

promise_test(t => {
  return testFourColorsDecode('four-colors.png', 'image/png');
}, 'Test PNG image decoding.');

promise_test(t => {
  return testFourColorsDecode('four-colors.avif', 'image/avif');
}, 'Test AVIF image decoding.');

promise_test(t => {
  return testFourColorsDecode('four-colors.webp', 'image/webp');
}, 'Test WEBP image decoding.');

promise_test(t => {
  return testFourColorsDecode('four-colors.gif', 'image/gif');
}, 'Test GIF image decoding.');
