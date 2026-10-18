// META: global=window,dedicatedworker

const width = 4;
const height = 2;

const formats = [
  {format: 'I420', planes: [{stride: 4, rows: 2}, {stride: 2, rows: 1}, {stride: 2, rows: 1}]},
  {format: 'I422', planes: [{stride: 4, rows: 2}, {stride: 2, rows: 2}, {stride: 2, rows: 2}]},
  {format: 'I444', planes: [{stride: 4, rows: 2}, {stride: 4, rows: 2}, {stride: 4, rows: 2}]},
];

function planesSize(planes) {
  return planes.reduce((total, plane) => total + plane.stride * plane.rows, 0);
}

function makeFrame(format, planes) {
  return new VideoFrame(new Uint8Array(planesSize(planes)), {
    format,
    timestamp: 0,
    codedWidth: width,
    codedHeight: height,
  });
}

for (const {format, planes} of formats) {
  test(t => {
    const frame = makeFrame(format, planes);
    t.add_cleanup(() => frame.close());
    assert_equals(frame.allocationSize(), planesSize(planes));
  }, `Test ${format} allocationSize()`);

  promise_test(async t => {
    const frame = makeFrame(format, planes);
    t.add_cleanup(() => frame.close());
    const buffer = new Uint8Array(frame.allocationSize());
    const layout = await frame.copyTo(buffer);
    assert_equals(layout.length, planes.length);
    let offset = 0;
    for (let i = 0; i < planes.length; ++i) {
      assert_equals(layout[i].offset, offset, `plane ${i} offset`);
      assert_equals(layout[i].stride, planes[i].stride, `plane ${i} stride`);
      offset += planes[i].stride * planes[i].rows;
    }
  }, `Test ${format} copyTo() layout`);
}

test(t => {
  const frame = makeFrame('I422', formats[1].planes);
  t.add_cleanup(() => frame.close());
  assert_equals(frame.allocationSize({rect: {x: 0, y: 1, width: 4, height: 1}}), 4 + 2 + 2);
  assert_throws_js(TypeError, () => frame.allocationSize({rect: {x: 1, y: 0, width: 2, height: 2}}));
}, 'Test I422 allows an odd rect.y and rejects an odd rect.x');

test(t => {
  const frame = makeFrame('I444', formats[2].planes);
  t.add_cleanup(() => frame.close());
  assert_equals(frame.allocationSize({rect: {x: 1, y: 1, width: 3, height: 1}}), 3 * 3);
}, 'Test I444 allows an odd rect offset');
