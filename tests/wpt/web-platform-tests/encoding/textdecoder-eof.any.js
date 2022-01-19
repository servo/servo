test(() => {
  assert_equals(new TextDecoder().decode(new Uint8Array([0xF0])), "\uFFFD");
  assert_equals(new TextDecoder().decode(new Uint8Array([0xF0, 0x9F])), "\uFFFD");
  assert_equals(new TextDecoder().decode(new Uint8Array([0xF0, 0x9F, 0x92])), "\uFFFD");
}, "TextDecoder end-of-queue handling");

test(() => {
  const decoder = new TextDecoder();
  decoder.decode(new Uint8Array([0xF0]), { stream: true });
  assert_equals(decoder.decode(), "\uFFFD");

  decoder.decode(new Uint8Array([0xF0]), { stream: true });
  decoder.decode(new Uint8Array([0x9F]), { stream: true });
  assert_equals(decoder.decode(), "\uFFFD");

  decoder.decode(new Uint8Array([0xF0, 0x9F]), { stream: true });
  assert_equals(decoder.decode(new Uint8Array([0x92])), "\uFFFD");
}, "TextDecoder end-of-queue handling using stream: true");
