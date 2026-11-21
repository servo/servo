// META: global=window,worker

// Test that `TextEncoder.encodeInto()` does not modify the buffer beyond the reported written length.
test(() => {
  const FILLER = 0x71; // "q"
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  for (const nonAscii of ["ä", "€"]) {
    for (let prefixLen = 0; prefixLen < 70; prefixLen++) {
      const source = "a".repeat(prefixLen) + nonAscii + "b".repeat(40);
      for (let destinationLength = 0; destinationLength <= source.length * 3; destinationLength++) {
        const destination = new Uint8Array(destinationLength).fill(FILLER);
        const { read, written } = encoder.encodeInto(source, destination);
        for (let i = written; i < destinationLength; i++) {
          assert_equals(destination[i], FILLER,
            `${nonAscii}, prefix length ${prefixLen}, destination length ${destinationLength}, written ${written}: byte at index ${i} modified`);
        }
        assert_equals(decoder.decode(destination.subarray(0, written)), source.slice(0, read),
          `${nonAscii}, prefix length ${prefixLen}, destination length ${destinationLength}: written bytes match read code units`);
      }
    }
  }
}, "encodeInto() leaves the destination beyond written unmodified");
