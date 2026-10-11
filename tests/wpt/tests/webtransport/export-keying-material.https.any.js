// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

const encoder = new TextEncoder();
const defaultLabel = encoder.encode('webtransport-exporter-label');
const defaultContext = encoder.encode('webtransport-exporter-context');

async function createTransport(t) {
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});
  await wt.ready;
  return wt;
}

promise_test(async t => {
  const wt = await createTransport(t);
  for (const length of [1, 32, 4096]) {
    const result =
        await wt.exportKeyingMaterial(defaultLabel, defaultContext, length);
    assert_true(result instanceof Uint8Array);
    assert_equals(result.byteLength, length);
  }
}, 'exportKeyingMaterial returns the requested number of bytes');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const result =
      await wt.exportKeyingMaterial(defaultLabel, defaultContext, 32);
  assert_true(result instanceof Uint8Array);
  assert_equals(result.byteLength, 32);
  await wt.ready;
}, 'exportKeyingMaterial can be called while connecting');

promise_test(async t => {
  const wt = await createTransport(t);
  const first = await wt.exportKeyingMaterial(defaultLabel, defaultContext, 32);
  const second =
      await wt.exportKeyingMaterial(defaultLabel, defaultContext, 32);
  assert_array_equals(first, second);
}, 'The same inputs on one session produce identical keying material');

promise_test(async t => {
  const wt = await createTransport(t);
  const first = await wt.exportKeyingMaterial(encoder.encode('first-label'),
                                              defaultContext, 32);
  const second = await wt.exportKeyingMaterial(encoder.encode('second-label'),
                                               defaultContext, 32);
  assert_not_equals(first.toString(), second.toString());
}, 'Different labels produce different keying material');

promise_test(async t => {
  const wt = await createTransport(t);
  const first = await wt.exportKeyingMaterial(
      defaultLabel, encoder.encode('first-context'), 32);
  const second = await wt.exportKeyingMaterial(
      defaultLabel, encoder.encode('second-context'), 32);
  assert_not_equals(first.toString(), second.toString());
}, 'Different contexts produce different keying material');

promise_test(async t => {
  const firstTransport = await createTransport(t);
  const secondTransport = await createTransport(t);
  const first = await firstTransport.exportKeyingMaterial(defaultLabel,
                                                          defaultContext, 32);
  const second = await secondTransport.exportKeyingMaterial(defaultLabel,
                                                            defaultContext, 32);
  assert_not_equals(first.toString(), second.toString());
}, 'Different sessions produce different keying material');

promise_test(async t => {
  const wt = await createTransport(t);
  const label = encoder.encode('buffer-source-label');
  const context = encoder.encode('buffer-source-context');
  const fromViews = await wt.exportKeyingMaterial(label, context, 32);
  const fromBuffers =
      await wt.exportKeyingMaterial(label.buffer, context.buffer, 32);
  assert_array_equals(fromViews, fromBuffers);
}, 'exportKeyingMaterial accepts ArrayBuffer and ArrayBufferView inputs');

promise_test(async t => {
  const wt = await createTransport(t);
  const labelBuffer = new Uint8Array(defaultLabel.byteLength + 4);
  const contextBuffer = new Uint8Array(defaultContext.byteLength + 6);
  labelBuffer.set(defaultLabel, 2);
  contextBuffer.set(defaultContext, 3);
  const label = new Uint8Array(labelBuffer.buffer, 2, defaultLabel.byteLength);
  const context =
      new Uint8Array(contextBuffer.buffer, 3, defaultContext.byteLength);

  const fromOffsetViews = await wt.exportKeyingMaterial(label, context, 32);
  const fromCopies =
      await wt.exportKeyingMaterial(defaultLabel, defaultContext, 32);
  assert_array_equals(fromOffsetViews, fromCopies);
}, 'exportKeyingMaterial respects non-zero ArrayBufferView offsets');

promise_test(async t => {
  const wt = await createTransport(t);
  const result =
      await wt.exportKeyingMaterial(new Uint8Array(0), new Uint8Array(0), 32);
  assert_true(result instanceof Uint8Array);
  assert_equals(result.byteLength, 32);
}, 'exportKeyingMaterial accepts empty label and context inputs');

promise_test(async t => {
  const wt = await createTransport(t);
  const result = await wt.exportKeyingMaterial(new Uint8Array(255),
                                               new Uint8Array(255), 32);
  assert_equals(result.byteLength, 32);
}, 'exportKeyingMaterial accepts 255-byte label and context inputs');

promise_test(async t => {
  const wt = await createTransport(t);
  await promise_rejects_js(
      t, RangeError,
      wt.exportKeyingMaterial(new Uint8Array(256), defaultContext, 32));
  await promise_rejects_js(
      t, RangeError,
      wt.exportKeyingMaterial(defaultLabel, new Uint8Array(256), 32));
}, 'exportKeyingMaterial rejects label and context inputs over 255 bytes');

promise_test(async t => {
  const wt = await createTransport(t);
  await promise_rejects_js(
      t, RangeError, wt.exportKeyingMaterial(defaultLabel, defaultContext, 0));
}, 'exportKeyingMaterial rejects a zero output length');

promise_test(async t => {
  const wt = await createTransport(t);
  await promise_rejects_js(
      t, RangeError,
      wt.exportKeyingMaterial(defaultLabel, defaultContext, 4097));
}, 'exportKeyingMaterial rejects an output length over 4096 bytes');

promise_test(async t => {
  const wt = await createTransport(t);
  await promise_rejects_js(t, TypeError, wt.exportKeyingMaterial(defaultLabel));
  await promise_rejects_js(
      t, TypeError, wt.exportKeyingMaterial(defaultLabel, defaultContext));
}, 'exportKeyingMaterial requires all three arguments');

promise_test(async t => {
  const wt = await createTransport(t);
  wt.close();
  await wt.closed;
  await promise_rejects_dom(
      t, 'InvalidStateError',
      wt.exportKeyingMaterial(defaultLabel, defaultContext, 32));
}, 'exportKeyingMaterial rejects after the session closes');
