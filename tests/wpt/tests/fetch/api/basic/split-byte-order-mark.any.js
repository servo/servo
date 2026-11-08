// META: title=Fetch: byte order marks split across chunks

// content-in-chunks.py sends each hex-encoded chunk on its own, so these check
// that decoding carries over from one chunk to the next.
function chunkedURL(chunks) {
  return "/xhr/resources/content-in-chunks.py?type=text/plain" +
      chunks.map(chunk => "&chunk=" + chunk).join("");
}

promise_test(async () => {
  const response = await fetch(chunkedURL(["ef", "bbbf50"]));
  assert_equals(await response.text(), "P");
}, "text() strips a UTF-8 byte order mark split across chunks");

promise_test(async () => {
  const response = await fetch(chunkedURL(["50", "efbbbf"]));
  assert_equals(await response.text(), "P\ufeff");
}, "text() does not strip a UTF-8 byte order mark in a later chunk");

promise_test(async () => {
  const response = await fetch(chunkedURL(["50c3", "a9"]));
  assert_equals(await response.text(), "P\u00e9");
}, "text() decodes a UTF-8 sequence split across chunks");

promise_test(async () => {
  const response = await fetch(chunkedURL(["ff", "fe5000"]));
  assert_equals(await response.text(), "\ufffd\ufffdP\u0000");
}, "text() does not use a UTF-16LE byte order mark split across chunks");

promise_test(async () => {
  const response = await fetch(chunkedURL(["ef", "bbbf31"]));
  assert_equals(await response.json(), 1);
}, "json() strips a UTF-8 byte order mark split across chunks");
