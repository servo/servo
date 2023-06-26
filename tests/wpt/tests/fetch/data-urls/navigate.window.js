// META: timeout=long
//
// Test some edge cases around navigation to data: URLs to ensure they use the same code path

[
  {
    input: "data:text/html,<script>parent.postMessage(1, '*')</script>",
    result: 1,
    name: "Nothing fancy",
  },
  {
    input: "data:text/html;base64,PHNjcmlwdD5wYXJlbnQucG9zdE1lc3NhZ2UoMiwgJyonKTwvc2NyaXB0Pg==",
    result: 2,
    name: "base64",
  },
  {
    input: "data:text/html;base64,PHNjcmlwdD5wYXJlbnQucG9zdE1lc3NhZ2UoNCwgJyonKTwvc2NyaXB0Pr+/",
    result: 4,
    name: "base64 with code points that differ from base64url"
  },
  {
    input: "data:text/html;base64,PHNjcml%09%20%20%0A%0C%0DwdD5wYXJlbnQucG9zdE1lc3NhZ2UoNiwgJyonKTwvc2NyaXB0Pg==",
    result: 6,
    name: "ASCII whitespace in the input is removed"
  }
].forEach(({ input, result, name }) => {
  // Use promise_test so they go sequentially
  promise_test(async t => {
    const event = await new Promise((resolve, reject) => {
      self.addEventListener("message", t.step_func(resolve), { once: true });
      const frame = document.body.appendChild(document.createElement("iframe"));
      t.add_cleanup(() => frame.remove());

      // The assumption is that postMessage() is quicker
      t.step_timeout(reject, 500);
      frame.src = input;
    });
    assert_equals(event.data, result);
  }, name);
});

// Failure cases
[
  {
    input: "data:text/html;base64,PHNjcmlwdD5wYXJlbnQucG9zdE1lc3NhZ2UoMywgJyonKTwvc2NyaXB0Pg=",
    name: "base64 with incorrect padding",
  },
  {
    input: "data:text/html;base64,PHNjcmlwdD5wYXJlbnQucG9zdE1lc3NhZ2UoNSwgJyonKTwvc2NyaXB0Pr-_",
    name: "base64url is not supported"
  },
  {
    input: "data:text/html;base64,%0BPHNjcmlwdD5wYXJlbnQucG9zdE1lc3NhZ2UoNywgJyonKTwvc2NyaXB0Pg==",
    name: "Vertical tab in the input leads to an error"
  }
].forEach(({ input, name }) => {
  // Continue to use promise_test so they go sequentially
  promise_test(async t => {
    const event = await new Promise((resolve, reject) => {
      self.addEventListener("message", t.step_func(reject), { once: true });
      const frame = document.body.appendChild(document.createElement("iframe"));
      t.add_cleanup(() => frame.remove());

      // The assumption is that postMessage() is quicker
      t.step_timeout(resolve, 500);
      frame.src = input;
    });
  }, name);
});

// I found some of the interesting code point cases above through brute force:
//
// for (i = 0; i < 256; i++) {
//   w(btoa("<script>parent.postMessage(5, '*')<\/script>" + String.fromCodePoint(i) + String.fromCodePoint(i)));
// }
