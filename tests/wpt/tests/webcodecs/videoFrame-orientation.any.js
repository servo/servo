// META: global=window,dedicatedworker

function make4x2VideoFrame(rotation, flip) {
  // y y r r
  // b b g g
  let data = new Uint8Array([
    255, 255, 0, 255,
    255, 255, 0, 255,
    255, 0, 0, 255,
    255, 0, 0, 255,
    0, 0, 255, 255,
    0, 0, 255, 255,
    0, 255, 0, 255,
    0, 255, 0, 255,
  ]);
  return new VideoFrame(data, {
    format: "RGBX",
    codedWidth: 4,
    codedHeight: 2,
    timestamp: 0,
    rotation,
    flip
  });
}

test(_ => {
  let frame = make4x2VideoFrame(-315, true);
  assert_equals(frame.visibleRect.width, 4, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 2, 'visibleRect.height');
  assert_equals(frame.rotation, 90, 'rotation');
  assert_equals(frame.flip, true, 'flip');
  assert_equals(frame.displayWidth, 2, 'displayWidth');
  assert_equals(frame.displayHeight, 4, 'displayHeight');

  let width = frame.displayWidth;
  let height = frame.displayHeight;
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d');
  ctx.drawImage(frame, 0, 0);
  frame.close();
  let data = ctx.getImageData(0, 0, width, height).data;

  function check_pixel(x, y, expected_color) {
    let tolerance = 2;
    assert_approx_equals(data[4*(width*y+x)], expected_color[0], tolerance);
    assert_approx_equals(data[4*(width*y+x)+1], expected_color[1], tolerance);
    assert_approx_equals(data[4*(width*y+x)+2], expected_color[2], tolerance);
  }

  // Expected rendering:
  //   y b
  //   y b
  //   r g
  //   r g
  check_pixel(0, 0, [255, 255, 0]);
  check_pixel(1, 0, [0, 0, 255]);
  check_pixel(0, 3, [255, 0, 0]);
  check_pixel(1, 3, [0, 255, 0]);
}, 'Test oriented VideoFrame from ArrayBuffer');

test(_ => {
  for (let baseRotation = 0; baseRotation < 360; baseRotation += 90) {
    for (let baseFlip = 0; baseFlip < 2; ++baseFlip) {
      let baseFrame = make4x2VideoFrame(baseRotation, !!baseFlip);
      for (let deltaRotation = 0; deltaRotation < 360; deltaRotation += 90) {
        for (let deltaFlip = 0; deltaFlip < 2; ++deltaFlip) {
          let deltaFrame = new VideoFrame(baseFrame, {
              rotation: deltaRotation,
              flip: !!deltaFlip
          });
          let appliedRotation = !!baseFlip ? 360 - deltaRotation
                                           : deltaRotation;
          let expectedRotation = (baseRotation + appliedRotation) % 360;
          let expectedFlip = !!(baseFlip ^ deltaFlip);
          assert_equals(deltaFrame.rotation, expectedRotation, 'rotation');
          assert_equals(deltaFrame.flip, expectedFlip, 'flip');
          deltaFrame.close();
        }
      }
      baseFrame.close();
    }
  }
}, 'Test combinations of rotation and flip');
