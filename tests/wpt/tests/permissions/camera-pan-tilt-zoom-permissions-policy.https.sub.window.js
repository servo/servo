'use strict';

const same_origin_src =
    '/permissions/resources/camera-pan-tilt-zoom-subframe.html';
const cross_origin_src =
    'https://{{domains[www]}}:{{ports[https][0]}}' + same_origin_src;

function query_camera_ptz(test, src, allow_attribute) {
  return new Promise((resolve, reject) => {
    let frame;
    const handler = test.step_func(evt => {
      if (frame && evt.source === frame.contentWindow) {
        window.removeEventListener('message', handler);
        frame.remove();
        if (evt.data.error) {
          reject(new Error(evt.data.error));
        } else {
          resolve(evt.data);
        }
      }
    });

    window.addEventListener('message', handler);

    frame = document.createElement('iframe');
    if (typeof allow_attribute !== 'undefined') {
      frame.allow = allow_attribute;
    }
    frame.src = src;
    document.body.appendChild(frame);
  });
}

promise_test(async t => {
  const result = await query_camera_ptz(t, same_origin_src);
  assert_equals(result.cameraState, 'prompt', 'camera');
  assert_equals(result.ptzState, 'prompt', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "prompt" in same-origin iframe by default.');

promise_test(async t => {
  const result = await query_camera_ptz(t, same_origin_src, 'camera');
  assert_equals(result.cameraState, 'prompt', 'camera');
  assert_equals(result.ptzState, 'prompt', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "prompt" with allow="camera" in same-origin iframe.');

promise_test(async t => {
  const result = await query_camera_ptz(t, same_origin_src, 'camera \'none\'');
  assert_equals(result.cameraState, 'denied', 'camera');
  assert_equals(result.ptzState, 'denied', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "denied" with allow="camera \'none\'" in same-origin iframe.');

promise_test(async t => {
  const result = await query_camera_ptz(t, cross_origin_src);
  assert_equals(result.cameraState, 'denied', 'camera');
  assert_equals(result.ptzState, 'denied', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "denied" in cross-origin iframe without allow="camera".');

promise_test(async t => {
  const result = await query_camera_ptz(t, cross_origin_src, 'camera');
  assert_equals(result.cameraState, 'prompt', 'camera');
  assert_equals(result.ptzState, 'prompt', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "prompt" with allow="camera" in cross-origin iframe.');

promise_test(async t => {
  const result = await query_camera_ptz(t, cross_origin_src, 'camera \'none\'');
  assert_equals(result.cameraState, 'denied', 'camera');
  assert_equals(result.ptzState, 'denied', 'camera PTZ');
}, 'Camera and camera PTZ permissions are "denied" with allow="camera \'none\'" in cross-origin iframe.');
