function toUInt32(pixelArray) {
  let p = pixelArray.data;
  return ((p[0] << 24) + (p[1] << 16) + (p[2] << 8) + p[3]) >>> 0;
}

function flipMatrix(m) {
  return m.map(row => row.reverse());
}

function rotateMatrix(m, count) {
  for (var i = 0; i < count; ++i)
    m = m[0].map((val, index) => m.map(row => row[index]).reverse());
  return m;
}

function testFourColorsDecodeBuffer(buffer, mimeType, options = {}) {
  var decoder = new ImageDecoder(
      {data: buffer, type: mimeType, preferAnimation: options.preferAnimation});
  return decoder.decode().then(result => {
    assert_equals(result.image.displayWidth, 320);
    assert_equals(result.image.displayHeight, 240);
    if (options.preferAnimation !== undefined) {
      assert_greater_than(decoder.tracks.length, 1);
      assert_equals(
          options.preferAnimation, decoder.tracks.selectedTrack.animated);
    }
    if (options.yuvFormat !== undefined)
      assert_equals(result.image.format, options.yuvFormat);
    if (options.tolerance === undefined)
      options.tolerance = 0;

    let canvas = new OffscreenCanvas(
        result.image.displayWidth, result.image.displayHeight);
    let ctx = canvas.getContext('2d');
    ctx.drawImage(result.image, 0, 0);

    let top_left = ctx.getImageData(0, 0, 1, 1);
    let top_right = ctx.getImageData(result.image.displayWidth - 1, 0, 1, 1);
    let bottom_left = ctx.getImageData(0, result.image.displayHeight - 1, 1, 1);
    let left_corner = ctx.getImageData(
        result.image.displayWidth - 1, result.image.displayHeight - 1, 1, 1);

    assert_array_approx_equals(
        top_left.data, [0xFF, 0xFF, 0x00, 0xFF], options.tolerance,
        'top left corner is yellow');
    assert_array_approx_equals(
        top_right.data, [0xFF, 0x00, 0x00, 0xFF], options.tolerance,
        'top right corner is red');
    assert_array_approx_equals(
        bottom_left.data, [0x00, 0x00, 0xFF, 0xFF], options.tolerance,
        'bottom left corner is blue');
    assert_array_approx_equals(
        left_corner.data, [0x00, 0xFF, 0x00, 0xFF], options.tolerance,
        'bottom right corner is green');
  });
}

function testFourColorDecodeWithExifOrientation(orientation, canvas) {
  return ImageDecoder.isTypeSupported('image/jpeg').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/jpeg not supported.');
    return fetch('four-colors.jpg')
        .then(response => {
          return response.arrayBuffer();
        })
        .then(buffer => {
          let u8buffer = new Uint8Array(buffer);
          u8buffer[0x1F] = orientation;  // Location derived via diff.
          let decoder = new ImageDecoder({data: u8buffer, type: 'image/jpeg'});
          return decoder.decode();
        })
        .then(result => {
          let respectOrientation = true;
          if (canvas)
            respectOrientation = canvas.style.imageOrientation != 'none';

          let expectedWidth = 320;
          let expectedHeight = 240;
          if (orientation > 4 && respectOrientation)
            [expectedWidth, expectedHeight] = [expectedHeight, expectedWidth];

          if (respectOrientation) {
            assert_equals(result.image.displayWidth, expectedWidth);
            assert_equals(result.image.displayHeight, expectedHeight);
          } else if (orientation > 4) {
            assert_equals(result.image.displayHeight, expectedWidth);
            assert_equals(result.image.displayWidth, expectedHeight);
          }

          if (!canvas) {
            canvas = new OffscreenCanvas(
                result.image.displayWidth, result.image.displayHeight);
          } else {
            canvas.width = expectedWidth;
            canvas.height = expectedHeight;
          }

          let ctx = canvas.getContext('2d');
          ctx.drawImage(result.image, 0, 0);

          let matrix = [
            [0xFFFF00FF, 0xFF0000FF],  // yellow, red
            [0x0000FFFF, 0x00FF00FF],  // blue, green
          ];
          if (respectOrientation) {
            switch (orientation) {
              case 1:  // kOriginTopLeft, default
                break;
              case 2:  // kOriginTopRight, mirror along y-axis
                matrix = flipMatrix(matrix);
                break;
              case 3:  // kOriginBottomRight, 180 degree rotation
                matrix = rotateMatrix(matrix, 2);
                break;
              case 4:  // kOriginBottomLeft, mirror along the x-axis
                matrix = flipMatrix(rotateMatrix(matrix, 2));
                break;
              case 5:  // kOriginLeftTop, mirror along x-axis + 270 degree CW
                       // rotation
                matrix = flipMatrix(rotateMatrix(matrix, 1));
                break;
              case 6:  // kOriginRightTop, 90 degree CW rotation
                matrix = rotateMatrix(matrix, 1);
                break;
              case 7:  // kOriginRightBottom, mirror along x-axis + 90 degree CW
                       // rotation
                matrix = flipMatrix(rotateMatrix(matrix, 3));
                break;
              case 8:  // kOriginLeftBottom, 270 degree CW rotation
                matrix = rotateMatrix(matrix, 3);
                break;
              default:
                assert_between_inclusive(
                    orientation, 1, 8, 'unknown image orientation');
                break;
            };
          }

          verifyFourColorsImage(expectedWidth, expectedHeight, ctx, matrix);
        });
  });
}

function verifyFourColorsImage(width, height, ctx, matrix) {
  if (!matrix) {
    matrix = [
      [0xFFFF00FF, 0xFF0000FF],  // yellow, red
      [0x0000FFFF, 0x00FF00FF],  // blue, green
    ];
  }

  let expectedTopLeft = matrix[0][0];
  let expectedTopRight = matrix[0][1];
  let expectedBottomLeft = matrix[1][0];
  let expectedBottomRight = matrix[1][1];

  let topLeft = toUInt32(ctx.getImageData(0, 0, 1, 1));
  let topRight = toUInt32(ctx.getImageData(width - 1, 0, 1, 1));
  let bottomLeft = toUInt32(ctx.getImageData(0, height - 1, 1, 1));
  let bottomRight = toUInt32(ctx.getImageData(width - 1, height - 1, 1, 1));

  assert_equals(topLeft, expectedTopLeft, 'top left corner');
  assert_equals(topRight, expectedTopRight, 'top right corner');
  assert_equals(bottomLeft, expectedBottomLeft, 'bottom left corner');
  assert_equals(bottomRight, expectedBottomRight, 'bottom right corner');
}
