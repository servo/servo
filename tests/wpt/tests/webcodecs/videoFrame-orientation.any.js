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

// Verifies various properties about a frame created frommake4x2VideoFrame()
// with flip=true and a 90 degree rotation.
function validateOrientedFrame(frame) {
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
}

test(_ => {
  let frame = make4x2VideoFrame(-315, true);
  validateOrientedFrame(frame);
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

test(_ => {
  let orig_frame = make4x2VideoFrame(0, false);
  let frame = new VideoFrame(orig_frame, {rotation: -315, flip: true});
  orig_frame.close();
  validateOrientedFrame(frame);
}, 'Test orientation of wrapped VideoFrame');

test(_ => {
  let frame = make4x2VideoFrame(undefined, undefined);
  assert_equals(frame.rotation, 0, 'rotation');
  assert_equals(frame.flip, false, 'flip');
  frame.close();
}, 'A VideoFrame is not rotated and not flipped by default');

test(_ => {
  // The rotation of the init is applied to the image before the flip of the base frame.
  let baseFrame = make4x2VideoFrame(90, true);
  let frame = new VideoFrame(baseFrame, {rotation: 90});
  assert_equals(frame.rotation, 0, 'rotation');
  assert_equals(frame.flip, true, 'flip');
  frame.close();
  baseFrame.close();
}, 'A rotation of the init is applied before the flip of the base frame');

test(_ => {
  let baseFrame = make4x2VideoFrame(90, true);
  let clone = baseFrame.clone();
  assert_equals(clone.rotation, 90, 'clone rotation');
  assert_equals(clone.flip, true, 'clone flip');
  let frame = new VideoFrame(clone, {rotation: 90, flip: true});
  assert_equals(frame.rotation, 0, 'derived rotation');
  assert_equals(frame.flip, false, 'derived flip');
  assert_equals(clone.rotation, 90, 'clone rotation after deriving');
  assert_equals(clone.flip, true, 'clone flip after deriving');
  assert_equals(baseFrame.rotation, 90, 'base rotation after deriving');
  assert_equals(baseFrame.flip, true, 'base flip after deriving');
  frame.close();
  clone.close();
  baseFrame.close();
}, 'Cloning an oriented VideoFrame keeps its orientation, and deriving from the clone does not change it');

test(_ => {
  let frame = make4x2VideoFrame(90, true);
  frame.close();
  assert_equals(frame.rotation, 0, 'rotation');
  assert_equals(frame.flip, false, 'flip');
}, 'A closed VideoFrame has no rotation and no flip');

promise_test(async t => {
  let data = new Uint8Array(4 * 2 * 4).map((_, i) => i * 7);
  let frame = new VideoFrame(data, {format: 'RGBA', codedWidth: 4, codedHeight: 2, timestamp: 0, rotation: 90, flip: true});
  t.add_cleanup(() => frame.close());
  let copy = new Uint8Array(frame.allocationSize());
  await frame.copyTo(copy);
  assert_array_equals(copy, data);
}, 'copyTo() of an oriented VideoFrame is not affected by its rotation and flip');
