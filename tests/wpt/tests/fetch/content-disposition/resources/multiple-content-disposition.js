// Shared by the HTTP/1.1 and HTTP/2 variants. See ../README.md.

const MULTIPLE_CONTENT_DISPOSITION_CASES = [
  {
    name: "identical",
    values: ['attachment; filename="a.txt"', 'attachment; filename="a.txt"'],
  },
  {
    name: "differing",
    values: ['inline; filename="a.txt"', 'attachment; filename="b.txt"'],
  },
];

// urlFor(values, name) returns a URL for a text/plain response with body
// "Test.\n" and `values` as repeated Content-Disposition field lines.
function runMultipleContentDispositionTests(protocol, urlFor) {
  for (const { name, values } of MULTIPLE_CONTENT_DISPOSITION_CASES) {
    promise_test(async () => {
      const response = await fetch(urlFor(values, name));
      assert_equals(response.status, 200);
      assert_equals(await response.text(), "Test.\n");
    }, `${protocol}: ${name} duplicate Content-Disposition headers are not a network error`);

    promise_test(async () => {
      const response = await fetch(urlFor(values, name));
      assert_equals(response.headers.get("content-disposition"), values.join(", "));
    }, `${protocol}: ${name} duplicate Content-Disposition headers are combined`);
  }
}
