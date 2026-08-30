// META: global=window,worker

// https://github.com/whatwg/compression/issues/84

// Brotli encoding of "Hello World" generated with large_window = true
// and lgwin = 25.
const LARGE_WINDOW_CHUNK = new Uint8Array([17, 25, 20, 0, 2, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 3]);

promise_test(async (t) => {
  const stream = new Blob([LARGE_WINDOW_CHUNK]).stream().pipeThrough(new DecompressionStream("brotli"));
  const text = await new Response(stream).text();
  assert_equals(text, "Hello World");
}, "DecompressionStream should support brotli large window");
