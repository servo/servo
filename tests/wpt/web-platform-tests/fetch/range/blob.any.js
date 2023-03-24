// META: script=/common/utils.js

const supportedBlobRange = [
  {
    name: "A simple blob range request.",
    data: ["A simple Hello, World! example"],
    type: "text/plain",
    range: "bytes=9-21",
    content_length: 13,
    content_range: "bytes 9-21/30",
    result: "Hello, World!",
  },
  {
    name: "A blob range request with no end.",
    data: ["Range with no end"],
    type: "text/plain",
    range: "bytes=11-",
    content_length: 6,
    content_range: "bytes 11-16/17",
    result: "no end",
  },
  {
    name: "A blob range request with no start.",
    data: ["Range with no start"],
    type: "text/plain",
    range: "bytes=-8",
    content_length: 8,
    content_range: "bytes 11-18/19",
    result: "no start",
  },
  {
    name: "A simple blob range request with whitespace.",
    data: ["A simple Hello, World! example"],
    type: "text/plain",
    range: "bytes= \t9-21",
    content_length: 13,
    content_range: "bytes 9-21/30",
    result: "Hello, World!",
  },
  {
    name: "Blob content with short content and a large range end",
    data: ["Not much here"],
    type: "text/plain",
    range: "bytes=4-100000000000",
    content_length: 9,
    content_range: "bytes 4-12/13",
    result: "much here",
  },
  {
    name: "Blob content with short content and a range end matching content length",
    data: ["Not much here"],
    type: "text/plain",
    range: "bytes=4-13",
    content_length: 9,
    content_range: "bytes 4-12/13",
    result: "much here",
  },
  {
    name: "Blob range with whitespace before and after hyphen",
    data: ["Valid whitespace #1"],
    type: "text/plain",
    range: "bytes=5 - 10",
    content_length: 6,
    content_range: "bytes 5-10/19",
    result: " white",
  },
  {
    name: "Blob range with whitespace after hyphen",
    data: ["Valid whitespace #2"],
    type: "text/plain",
    range: "bytes=-\t 5",
    content_length: 5,
    content_range: "bytes 14-18/19",
    result: "ce #2",
  },
  {
    name: "Blob range with whitespace around equals sign",
    data: ["Valid whitespace #3"],
    type: "text/plain",
    range: "bytes \t =\t 6-",
    content_length: 13,
    content_range: "bytes 6-18/19",
    result: "whitespace #3",
  },
];

const unsupportedBlobRange = [
  {
    name: "Blob range with no value",
    data: ["Blob range should have a value"],
    type: "text/plain",
    range: "",
  },
  {
    name: "Blob range with incorrect range header",
    data: ["A"],
    type: "text/plain",
    range: "byte=0-"
  },
  {
    name: "Blob range with incorrect range header #2",
    data: ["A"],
    type: "text/plain",
    range: "bytes"
  },
  {
    name: "Blob range with incorrect range header #3",
    data: ["A"],
    type: "text/plain",
    range: "bytes\t \t"
  },
  {
    name: "Blob range request with multiple range values",
    data: ["Multiple ranges are not currently supported"],
    type: "text/plain",
    range: "bytes=0-5,15-",
  },
  {
    name: "Blob range request with multiple range values and whitespace",
    data: ["Multiple ranges are not currently supported"],
    type: "text/plain",
    range: "bytes=0-5, 15-",
  },
  {
    name: "Blob range request with trailing comma",
    data: ["Range with invalid trailing comma"],
    type: "text/plain",
    range: "bytes=0-5,",
  },
  {
    name: "Blob range with no start or end",
    data: ["Range with no start or end"],
    type: "text/plain",
    range: "bytes=-",
  },
  {
    name: "Blob range request with short range end",
    data: ["Range end should be greater than range start"],
    type: "text/plain",
    range: "bytes=10-5",
  },
  {
    name: "Blob range start should be an ASCII digit",
    data: ["Range start must be an ASCII digit"],
    type: "text/plain",
    range: "bytes=x-5",
  },
  {
    name: "Blob range should have a dash",
    data: ["Blob range should have a dash"],
    type: "text/plain",
    range: "bytes=5",
  },
  {
    name: "Blob range end should be an ASCII digit",
    data: ["Range end must be an ASCII digit"],
    type: "text/plain",
    range: "bytes=5-x",
  },
  {
    name: "Blob range should include '-'",
    data: ["Range end must include '-'"],
    type: "text/plain",
    range: "bytes=x",
  },
  {
    name: "Blob range should include '='",
    data: ["Range end must include '='"],
    type: "text/plain",
    range: "bytes 5-",
  },
  {
    name: "Blob range should include 'bytes='",
    data: ["Range end must include 'bytes='"],
    type: "text/plain",
    range: "5-",
  },
  {
    name: "Blob content with short content and a large range start",
    data: ["Not much here"],
    type: "text/plain",
    range: "bytes=100000-",
  },
  {
    name: "Blob content with short content and a range start matching the content length",
    data: ["Not much here"],
    type: "text/plain",
    range: "bytes=13-",
  },
];

supportedBlobRange.forEach(({ name, data, type, range, content_length, content_range, result }) => {
  promise_test(async t => {
    const blob = new Blob(data, { "type" : type });
    const blobURL = URL.createObjectURL(blob);
    t.add_cleanup(() => URL.revokeObjectURL(blobURL));
    const resp = await fetch(blobURL, {
      "headers": {
        "Range": range
      }
    });
    assert_equals(resp.status, 206, "HTTP status is 206");
    assert_equals(resp.type, "basic", "response type is basic");
    assert_equals(resp.headers.get("Content-Type"), type, "Content-Type is " + resp.headers.get("Content-Type"));
    assert_equals(resp.headers.get("Content-Length"), content_length.toString(), "Content-Length is " + resp.headers.get("Content-Length"));
    assert_equals(resp.headers.get("Content-Range"), content_range, "Content-Range is " + resp.headers.get("Content-Range"));
    const text = await resp.text();
    assert_equals(text, result, "Response's body is correct");
  }, name);
});

unsupportedBlobRange.forEach(({ name, data, type, range }) => {
  promise_test(t => {
    const blob = new Blob(data, { "type" : type });
    const blobURL = URL.createObjectURL(blob);
    t.add_cleanup(() => URL.revokeObjectURL(blobURL));
    const promise = fetch(blobURL, {
      "headers": {
        "Range": range
      }
    });
    return promise_rejects_js(t, TypeError, promise);
  }, name);
});
