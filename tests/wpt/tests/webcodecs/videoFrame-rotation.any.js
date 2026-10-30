// META: global=window,dedicatedworker

// Frames are 4x2 and made of RGBA pixels, so the display size of a frame is
// 4x2, or 2x4 when it is rotated by 90 or 270 degrees.
const kWidth = 4;
const kHeight = 2;

function makeData() {
  const data = new Uint8Array(kWidth * kHeight * 4);
  for (let i = 0; i < data.length; ++i)
    data[i] = i * 7;
  return data;
}

function makeFrame(rotation) {
  const init = {
    format: 'RGBA',
    codedWidth: kWidth,
    codedHeight: kHeight,
    timestamp: 1234,
  };
  if (rotation !== undefined)
    init.rotation = rotation;
  return new VideoFrame(makeData(), init);
}

function expectedDisplaySize(rotation) {
  return (rotation % 180) ? {width: kHeight, height: kWidth} : {width: kWidth, height: kHeight};
}

function verifyFrame(frame, rotation, message) {
  const displaySize = expectedDisplaySize(rotation);
  assert_equals(frame.rotation, rotation, `${message}: rotation`);
  assert_equals(frame.displayWidth, displaySize.width, `${message}: displayWidth`);
  assert_equals(frame.displayHeight, displaySize.height, `${message}: displayHeight`);
  assert_equals(frame.codedWidth, kWidth, `${message}: codedWidth`);
  assert_equals(frame.codedHeight, kHeight, `${message}: codedHeight`);
  assert_equals(frame.visibleRect.width, kWidth, `${message}: visibleRect.width`);
  assert_equals(frame.visibleRect.height, kHeight, `${message}: visibleRect.height`);
}

test(() => {
  const frame = makeFrame();
  assert_equals(frame.rotation, 0);
  frame.close();
}, 'A VideoFrame has no rotation by default');

for (const rotation of [0, 90, 180, 270]) {
  test(() => {
    const frame = makeFrame(rotation);
    verifyFrame(frame, rotation, 'frame');
    frame.close();
  }, `A VideoFrame created from data with a rotation of ${rotation}`);
}

for (const baseRotation of [0, 90, 180, 270]) {
  for (const deltaRotation of [0, 90, 180, 270]) {
    test(() => {
      const baseFrame = makeFrame(baseRotation);
      const frame = new VideoFrame(baseFrame, {rotation: deltaRotation});
      verifyFrame(frame, (baseRotation + deltaRotation) % 360, 'derived frame');
      // Creating a frame does not change the frame it is created from.
      verifyFrame(baseFrame, baseRotation, 'base frame');
      frame.close();
      baseFrame.close();
    }, `A VideoFrame created from a VideoFrame with a rotation of ${baseRotation} and an init rotation of ${deltaRotation}`);
  }
}

test(() => {
  const baseFrame = makeFrame(90);
  const frame = new VideoFrame(baseFrame);
  verifyFrame(frame, 90, 'derived frame');
  frame.close();
  baseFrame.close();
}, 'A VideoFrame created from a rotated VideoFrame without an init rotation keeps the rotation');

test(() => {
  const baseFrame = makeFrame(270);
  const frame = new VideoFrame(baseFrame, {});
  verifyFrame(frame, 270, 'derived frame');
  frame.close();
  baseFrame.close();
}, 'A VideoFrame created from a rotated VideoFrame with an empty init keeps the rotation');

test(() => {
  const baseFrame = makeFrame(0);
  const frames = [90, 180, 270].map(rotation => new VideoFrame(baseFrame, {rotation}));
  assert_array_equals(frames.map(frame => frame.rotation), [90, 180, 270]);
  assert_equals(baseFrame.rotation, 0);
  frames.forEach(frame => frame.close());
  baseFrame.close();
}, 'Several VideoFrames with different rotations can be created from the same VideoFrame');

test(() => {
  const baseFrame = makeFrame(90);
  const derivedFrame = new VideoFrame(baseFrame, {rotation: 90});
  baseFrame.close();
  // The derived frame stays valid and rotated after closing the frame it was created from.
  verifyFrame(derivedFrame, 180, 'derived frame');
  derivedFrame.close();
}, 'A rotated VideoFrame created from another VideoFrame outlives it');

test(() => {
  const baseFrame = makeFrame(90);
  const firstFrame = new VideoFrame(baseFrame, {rotation: 90});
  const secondFrame = new VideoFrame(firstFrame, {rotation: 90});
  verifyFrame(secondFrame, 270, 'second frame');
  verifyFrame(firstFrame, 180, 'first frame');
  verifyFrame(baseFrame, 90, 'base frame');
  secondFrame.close();
  firstFrame.close();
  baseFrame.close();
}, 'Rotations accumulate when creating a VideoFrame from a VideoFrame from a VideoFrame');

// The rotation is rounded to the nearest multiple of 90, ties towards positive infinity, then
// brought in [0, 360).
const kRotations = [
  // Multiples of 90 and values around the rounding limits.
  [-315, 90], [-360, 0], [-90, 270], [-45, 0], [-135, 270], [-1, 0], [44, 0], [45, 90],
  [89.9, 90], [135, 180], [315, 0], [359, 0], [360, 0], [450, 90], [720, 0],
  // Fractional values and values with several full turns.
  [0.5, 0], [-0.5, 0], [1080.5, 0], [-1080.5, 0], [1124.99, 0], [-1124.99, 0], [1125, 90],
  [-1125, 0], [123456.789, 0], [1e6, 270], [-1e6, 90],
  // Values for which the double precision is not enough to represent every integer.
  [2 ** 53, 0], [2 ** 53 + 2, 0], [1e15, 270], [1e16, 270], [1e17, 270], [1e20, 270],
  [1e300, 0], [-1e300, 0], [Number.MAX_VALUE, 90], [-Number.MAX_VALUE, 270],
  [Number.MIN_VALUE, 0], [-Number.MIN_VALUE, 0],
];

for (const [rotation, expected] of kRotations) {
  test(() => {
    const frame = makeFrame(rotation);
    verifyFrame(frame, expected, 'frame');
    frame.close();
  }, `An init rotation of ${rotation} gives a VideoFrame rotation of ${expected}`);

  test(() => {
    const baseFrame = makeFrame(90);
    const frame = new VideoFrame(baseFrame, {rotation});
    verifyFrame(frame, (90 + expected) % 360, 'derived frame');
    frame.close();
    baseFrame.close();
  }, `A VideoFrame created from a VideoFrame rotated by 90 with an init rotation of ${rotation}`);
}

for (const rotation of [NaN, Infinity, -Infinity]) {
  test(() => {
    const baseFrame = makeFrame(0);
    assert_throws_js(TypeError, () => new VideoFrame(baseFrame, {rotation}));
    baseFrame.close();
  }, `A VideoFrame cannot be created from a VideoFrame with an init rotation of ${rotation}`);
}

test(() => {
  const baseFrame = makeFrame(90);
  const frame = new VideoFrame(baseFrame, {rotation: 90, timestamp: 5678, duration: 10});
  verifyFrame(frame, 180, 'derived frame');
  assert_equals(frame.timestamp, 5678, 'timestamp');
  assert_equals(frame.duration, 10, 'duration');
  assert_equals(baseFrame.timestamp, 1234, 'base frame timestamp');
  frame.close();
  baseFrame.close();
}, 'A VideoFrame created from a VideoFrame can change both rotation and timestamp');

test(() => {
  const baseFrame = makeFrame(0);
  const frame = new VideoFrame(baseFrame, {rotation: 90, displayWidth: 20, displayHeight: 10});
  assert_equals(frame.rotation, 90, 'rotation');
  assert_equals(frame.displayWidth, 20, 'displayWidth');
  assert_equals(frame.displayHeight, 10, 'displayHeight');
  frame.close();
  baseFrame.close();
}, 'An explicit display size is not swapped by the init rotation');

test(() => {
  const baseFrame = makeFrame(90);
  const frame = new VideoFrame(baseFrame, {rotation: 90, visibleRect: {x: 0, y: 0, width: 2, height: 2}});
  assert_equals(frame.rotation, 180, 'rotation');
  assert_equals(frame.visibleRect.width, 2, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 2, 'visibleRect.height');
  assert_equals(frame.displayWidth, 2, 'displayWidth');
  assert_equals(frame.displayHeight, 2, 'displayHeight');
  frame.close();
  baseFrame.close();
}, 'A VideoFrame created from a rotated VideoFrame can also change the visible rect');

test(() => {
  const baseFrame = makeFrame(180);
  const clone = baseFrame.clone();
  verifyFrame(clone, 180, 'clone');
  const derivedFrame = new VideoFrame(clone, {rotation: 90});
  verifyFrame(derivedFrame, 270, 'derived frame');
  verifyFrame(clone, 180, 'clone after deriving');
  verifyFrame(baseFrame, 180, 'base frame after deriving');
  derivedFrame.close();
  clone.close();
  baseFrame.close();
}, 'Cloning a rotated VideoFrame keeps its rotation, and deriving from the clone does not change it');

test(() => {
  const frame = makeFrame(90);
  frame.close();
  assert_equals(frame.rotation, 0);
}, 'A closed VideoFrame has a rotation of 0');

test(() => {
  const baseFrame = makeFrame(90);
  baseFrame.close();
  assert_throws_dom('InvalidStateError', () => new VideoFrame(baseFrame, {rotation: 90}));
}, 'A VideoFrame cannot be created from a closed VideoFrame with a rotation');

// copyTo() gives the pixels of the frame, whatever its rotation.
promise_test(async t => {
  const expected = makeData();
  for (const rotation of [0, 90, 180, 270]) {
    const baseFrame = makeFrame(0);
    const frame = new VideoFrame(baseFrame, {rotation});
    const copy = new Uint8Array(frame.allocationSize());
    const layout = await frame.copyTo(copy);
    assert_equals(layout.length, 1, `layout length with a rotation of ${rotation}`);
    assert_array_equals(copy, expected, `copy with a rotation of ${rotation}`);
    frame.close();
    baseFrame.close();
  }
}, 'copyTo() of a VideoFrame created from another VideoFrame is not affected by its rotation');
