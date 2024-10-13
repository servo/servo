// META: timeout=long
// META: variant=?globalScope=window
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

'use strict';

test(() => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const frame_window = iframe.contentWindow;

  iframe.remove();
  assert_equals(undefined, frame_window.PressureObserver);
}, 'PressureObserver constructor does not exist in detached iframes');

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const frame_window = iframe.contentWindow;

  const observer = new frame_window.PressureObserver(() => {});
  const iframe_DOMException = frame_window.DOMException;

  iframe.remove();

  // Calling observe() from a detached iframe should fail but not crash.
  await promise_rejects_dom(t, 'NotSupportedError', iframe_DOMException,
                            observer.observe('cpu'));
}, 'PressureObserver.observe() on detached frame rejects');

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const frame_window = iframe.contentWindow;

  const observer = new frame_window.PressureObserver(() => {});

  await observer.observe('cpu');

  iframe.remove();

  // Calling disconnect() from a detached iframe should not crash.
  observer.disconnect();
}, 'PressureObserver.disconnect() on detached frame returns');

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const frame_window = iframe.contentWindow;

  const observer = new frame_window.PressureObserver(() => {});
  const iframe_DOMException = frame_window.DOMException;

  // await is intentionally not used here. We want to remove the iframe while
  // the returned Promise settles.
  observer.observe('cpu');
  iframe.remove();

  // Establish an observer and wait for changes in the main frame. This should
  // keep the test running long enough to catch any crash from the observe()
  // call in the removed iframe's PressureObserver.
  const changes = await new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
    update_virtual_pressure_source('cpu', 'critical').catch(reject);
  });
  assert_equals(changes[0].state, 'critical');
}, 'Detaching frame while PressureObserver.observe() settles');

pressure_test(async t => {
  await create_virtual_pressure_source('cpu');
  t.add_cleanup(async () => {
    await remove_virtual_pressure_source('cpu');
  });

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const frame_window = iframe.contentWindow;
  const observer = new frame_window.PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });

  await observer.observe('cpu');
  const updatePromise = update_virtual_pressure_source('cpu', 'critical');
  iframe.remove();
  await updatePromise;

  return new Promise(resolve => t.step_timeout(resolve, 3000));
}, 'PressureObserver on detached frame returns with no callback');

mark_as_done();
