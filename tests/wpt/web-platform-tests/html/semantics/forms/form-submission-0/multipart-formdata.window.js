// META: script=enctypes-helper.js

// Form submissions in multipart/form-data are also tested in
// /FileAPI/file/send-file*

const form = formSubmissionTemplate(
  "multipart/form-data",
  ({ name, filename, value }, serialized) => {
    let headers;
    if (filename === undefined) {
      headers = [`Content-Disposition: form-data; name="${name}"`];
    } else {
      headers = [
        `Content-Disposition: form-data; name="${name}"; filename="${filename}"`,
        "Content-Type: text/plain",
      ];
    }

    const boundary = serialized.split("\r\n")[0];

    return [
      boundary,
      ...headers,
      "",
      value,
      boundary + "--",
      "",
    ].join("\r\n");
  },
);

form({
  name: "basic",
  value: "test",
  expected: {
    name: "basic",
    value: "test",
  },
  description: "Basic test",
});

form({
  name: "basic",
  value: new File([], "file-test.txt", { type: "text/plain" }),
  expected: {
    name: "basic",
    filename: "file-test.txt",
    value: "",
  },
  description: "Basic File test",
});

form({
  name: "a\0b",
  value: "c",
  expected: {
    name: "a\0b",
    value: "c",
  },
  description: "0x00 in name",
});

form({
  name: "a",
  value: "b\0c",
  expected: {
    name: "a",
    value: "b\0c",
  },
  description: "0x00 in value",
});

form({
  name: "a",
  value: new File([], "b\0c", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b\0c",
    value: "",
  },
  description: "0x00 in filename",
});

form({
  name: "a\nb",
  value: "c",
  expected: {
    name: "a%0D%0Ab",
    value: "c",
  },
  description: "\\n in name",
});

form({
  name: "a\rb",
  value: "c",
  expected: {
    name: "a%0D%0Ab",
    value: "c",
  },
  description: "\\r in name",
});

form({
  name: "a\r\nb",
  value: "c",
  expected: {
    name: "a%0D%0Ab",
    value: "c",
  },
  description: "\\r\\n in name",
});

form({
  name: "a\n\rb",
  value: "c",
  expected: {
    name: "a%0D%0A%0D%0Ab",
    value: "c",
  },
  description: "\\n\\r in name",
});

form({
  name: "a",
  value: "b\nc",
  expected: {
    name: "a",
    value: "b\r\nc",
  },
  description: "\\n in value",
});

form({
  name: "a",
  value: "b\rc",
  expected: {
    name: "a",
    value: "b\r\nc",
  },
  description: "\\r in value",
});

form({
  name: "a",
  value: "b\r\nc",
  expected: {
    name: "a",
    value: "b\r\nc",
  },
  description: "\\r\\n in value",
});

form({
  name: "a",
  value: "b\n\rc",
  expected: {
    name: "a",
    value: "b\r\n\r\nc",
  },
  description: "\\n\\r in value",
});

form({
  name: "a",
  value: new File([], "b\nc", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b%0Ac",
    value: "",
  },
  description: "\\n in filename",
});

form({
  name: "a",
  value: new File([], "b\rc", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b%0Dc",
    value: "",
  },
  description: "\\r in filename",
});

form({
  name: "a",
  value: new File([], "b\r\nc", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b%0D%0Ac",
    value: "",
  },
  description: "\\r\\n in filename",
});

form({
  name: "a",
  value: new File([], "b\n\rc", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b%0A%0Dc",
    value: "",
  },
  description: "\\n\\r in filename",
});

form({
  name: 'a"b',
  value: "c",
  expected: {
    name: "a%22b",
    value: "c",
  },
  description: "double quote in name",
});

form({
  name: "a",
  value: 'b"c',
  expected: {
    name: "a",
    value: 'b"c',
  },
  description: "double quote in value",
});

form({
  name: "a",
  value: new File([], 'b"c', { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b%22c",
    value: "",
  },
  description: "double quote in filename",
});

form({
  name: "a'b",
  value: "c",
  expected: {
    name: "a'b",
    value: "c",
  },
  description: "single quote in name",
});

form({
  name: "a",
  value: "b'c",
  expected: {
    name: "a",
    value: "b'c",
  },
  description: "single quote in value",
});

form({
  name: "a",
  value: new File([], "b'c", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b'c",
    value: "",
  },
  description: "single quote in filename",
});

form({
  name: "a\\b",
  value: "c",
  expected: {
    name: "a\\b",
    value: "c",
  },
  description: "backslash in name",
});

form({
  name: "a",
  value: "b\\c",
  expected: {
    name: "a",
    value: "b\\c",
  },
  description: "backslash in value",
});

form({
  name: "a",
  value: new File([], "b\\c", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "b\\c",
    value: "",
  },
  description: "backslash in filename",
});

form({
  name: "áb",
  value: "ç",
  expected: {
    name: "\xC3\xA1b",
    value: "\xC3\xA7",
  },
  description: "non-ASCII in name and value",
});

form({
  name: "a",
  value: new File([], "ə.txt", { type: "text/plain" }),
  expected: {
    name: "a",
    filename: "\xC9\x99.txt",
    value: "",
  },
  description: "non-ASCII in filename",
});

form({
  name: "aəb",
  value: "c\uFFFDd",
  formEncoding: "windows-1252",
  expected: {
    name: "a&#601;b",
    value: "c&#65533;d",
  },
  description: "characters not in encoding in name and value",
});

form({
  name: "á",
  value: new File([], "💩", { type: "text/plain" }),
  formEncoding: "windows-1252",
  expected: {
    name: "\xE1",
    filename: "&#128169;",
    value: "",
  },
  description: "character not in encoding in filename",
});
