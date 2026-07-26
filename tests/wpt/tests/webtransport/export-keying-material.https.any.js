// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-WebTransport-test');
  const result = await wt.exportKeyingMaterial(label);

  assert_true(result instanceof Uint8Array, 'Result should be a Uint8Array');
  assert_greater_than(result.byteLength, 0, 'Result should not be empty');
}, 'exportKeyingMaterial with only label returns non-empty Uint8Array');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const encoder = new TextEncoder();
  const label = encoder.encode('EXPORTER-WebTransport-test-context');
  const context = encoder.encode('test-context-value');
  const result = await wt.exportKeyingMaterial(label, context);

  assert_true(result instanceof Uint8Array, 'Result should be a Uint8Array');
  assert_greater_than(result.byteLength, 0, 'Result should not be empty');
}, 'exportKeyingMaterial with label and context returns non-empty Uint8Array');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const encoder = new TextEncoder();
  const label1 = encoder.encode('EXPORTER-WebTransport-label1');
  const label2 = encoder.encode('EXPORTER-WebTransport-label2');

  const result1 = await wt.exportKeyingMaterial(label1);
  const result2 = await wt.exportKeyingMaterial(label2);

  assert_true(result1 instanceof Uint8Array, 'Result 1 should be a Uint8Array');
  assert_true(result2 instanceof Uint8Array, 'Result 2 should be a Uint8Array');
  assert_equals(result1.byteLength, result2.byteLength, 'Results should have same length');

  assert_false(
    result1.length === result2.length &&
    result1.every((v, i) => v === result2[i]),
    'Different labels should produce different keying material');
}, 'Different labels produce different keying material');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const encoder = new TextEncoder();
  const label = encoder.encode('EXPORTER-WebTransport-context-test');
  const context1 = encoder.encode('context1');
  const context2 = encoder.encode('context2');

  const result1 = await wt.exportKeyingMaterial(label, context1);
  const result2 = await wt.exportKeyingMaterial(label, context2);

  assert_true(result1 instanceof Uint8Array, 'Result 1 should be a Uint8Array');
  assert_true(result2 instanceof Uint8Array, 'Result 2 should be a Uint8Array');
  assert_equals(result1.byteLength, result2.byteLength, 'Results should have same length');

  assert_false(
    result1.length === result2.length &&
    result1.every((v, i) => v === result2[i]),
    'Different contexts should produce different keying material');
}, 'Different contexts produce different keying material');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-WebTransport-deterministic');
  const result1 = await wt.exportKeyingMaterial(label);
  const result2 = await wt.exportKeyingMaterial(label);

  assert_true(result1 instanceof Uint8Array, 'Result 1 should be a Uint8Array');
  assert_true(result2 instanceof Uint8Array, 'Result 2 should be a Uint8Array');
  assert_equals(result1.byteLength, result2.byteLength, 'Results should have same length');

  assert_array_equals(result1, result2);
}, 'Same label on same connection produces identical keying material');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  wt.close();
  await wt.closed;

  const label = new TextEncoder().encode('EXPORTER-test');
  await promise_rejects_dom(t, 'InvalidStateError',
    wt.exportKeyingMaterial(label),
    'exportKeyingMaterial should reject on closed connection');
}, 'exportKeyingMaterial rejects with InvalidStateError on closed connection');

promise_test(async t => {
  const wt1 = new WebTransport(webtransport_url('echo.py'));
  const wt2 = new WebTransport(webtransport_url('echo.py'));
  await wt1.ready;
  await wt2.ready;

  t.add_cleanup(() => { wt1.close(); wt2.close();});
  wt1.closed.catch(() => {});
  wt2.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-WebTransport-different-connections');
  const result1 = await wt1.exportKeyingMaterial(label);
  const result2 = await wt2.exportKeyingMaterial(label);

  assert_true(result1 instanceof Uint8Array, 'Result 1 should be a Uint8Array');
  assert_true(result2 instanceof Uint8Array, 'Result 2 should be a Uint8Array');
  assert_equals(result1.byteLength, result2.byteLength, 'Results should have same length');

  assert_false(
    result1.length === result2.length &&
    result1.every((v, i) => v === result2[i]),
    'Different connections should produce different keying material');
}, 'Different connections produce different keying material with same label');


promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-WebTransport-typed-array-test');
  const resultFromUint8Array = await wt.exportKeyingMaterial(new Uint8Array(label));
  const resultFromArrayBuffer = await wt.exportKeyingMaterial(label.buffer);

  assert_true(resultFromUint8Array instanceof Uint8Array, 'Result from Uint8Array should be Uint8Array');
  assert_true(resultFromArrayBuffer instanceof Uint8Array, 'Result from ArrayBuffer should be Uint8Array');
  assert_equals(resultFromUint8Array.byteLength, resultFromArrayBuffer.byteLength,
    'Results should have same length');

  assert_array_equals(resultFromUint8Array, resultFromArrayBuffer);
}, 'exportKeyingMaterial accepts both ArrayBufferView and ArrayBuffer for label');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new Uint8Array(256);
  await promise_rejects_js(t, RangeError,
    wt.exportKeyingMaterial(label),
    'exportKeyingMaterial should throw RangeError for label > 255 bytes');
}, 'exportKeyingMaterial throws RangeError when label exceeds 255 bytes');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-test');
  const context = new Uint8Array(256);
  await promise_rejects_js(t, RangeError,
    wt.exportKeyingMaterial(label, context),
    'exportKeyingMaterial should throw RangeError for context > 255 bytes');
}, 'exportKeyingMaterial throws RangeError when context exceeds 255 bytes');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  // The HKDF-Expand-Label info structure contains:
  // - 2 bytes: output length
  // - 1 byte: label length
  // - 6 bytes: "tls13 " prefix
  // - N bytes: application label
  // - 1 byte: hash length
  // - H bytes: hash value
  //
  // For SHA-256 (32 bytes hash):
  // - Overhead: 2 + 1 + 6 + 1 + 32 = 42 bytes
  // - Max label: 256 - 42 = 214 bytes
  //
  // For SHA-384 (48 bytes hash):
  // - Overhead: 2 + 1 + 6 + 1 + 48 = 58 bytes
  // - Max label: 256 - 58 = 198 bytes
  //
  // A 200-byte label with SHA-384 requires 258 bytes, exceeding the buffer.
  // Since WebTransport connections may use either SHA-256 or SHA-384 cipher
  // suites, the safe maximum label length for testing is 198 bytes to work
  // with all configurations

  const prefix = 'EXPORTER-WebTransport-';
  const padding = 'A'.repeat(198 - prefix.length);
  const labelStr = prefix + padding;
  const label = new TextEncoder().encode(labelStr);
  assert_equals(label.byteLength, 198, 'Label should be 198 bytes');
  const result = await wt.exportKeyingMaterial(label);

  assert_true(result instanceof Uint8Array, 'Result should be a Uint8Array');
  assert_greater_than(result.byteLength, 0, 'Result should not be empty');
}, 'exportKeyingMaterial accepts long label up to 198 bytes');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});

  const label = new TextEncoder().encode('EXPORTER-test');
  const contextStr = 'B'.repeat(200);
  const context = new TextEncoder().encode(contextStr);
  assert_equals(context.byteLength, 200, 'Context should be 200 bytes');
  const result = await wt.exportKeyingMaterial(label, context);

  assert_true(result instanceof Uint8Array, 'Result should be a Uint8Array');
  assert_greater_than(result.byteLength, 0, 'Result should not be empty');
}, 'exportKeyingMaterial accepts long context up to 255 bytes');
