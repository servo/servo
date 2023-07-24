// META: script=enctypes-helper.js

const formTest = formSubmissionTemplate("application/x-www-form-urlencoded");

formTest({
  name: "basic",
  value: "test",
  expected: "basic=test",
  description: "Basic test",
});

formTest({
  name: "basic",
  value: new File([], "file-test.txt"),
  expected: "basic=file-test.txt",
  description: "Basic File test",
});

formTest({
  name: "a\0b",
  value: "c",
  expected: "a%00b=c",
  description: "0x00 in name",
});

formTest({
  name: "a",
  value: "b\0c",
  expected: "a=b%00c",
  description: "0x00 in value",
});

formTest({
  name: "a",
  value: new File([], "b\0c"),
  expected: "a=b%00c",
  description: "0x00 in filename",
});

formTest({
  name: "a\nb",
  value: "c",
  expected: "a%0D%0Ab=c",
  description: "\\n in name",
});

formTest({
  name: "a\rb",
  value: "c",
  expected: "a%0D%0Ab=c",
  description: "\\r in name",
});

formTest({
  name: "a\r\nb",
  value: "c",
  expected: "a%0D%0Ab=c",
  description: "\\r\\n in name",
});

formTest({
  name: "a\n\rb",
  value: "c",
  expected: "a%0D%0A%0D%0Ab=c",
  description: "\\n\\r in name",
});

formTest({
  name: "a",
  value: "b\nc",
  expected: "a=b%0D%0Ac",
  description: "\\n in value",
});

formTest({
  name: "a",
  value: "b\rc",
  expected: "a=b%0D%0Ac",
  description: "\\r in value",
});

formTest({
  name: "a",
  value: "b\r\nc",
  expected: "a=b%0D%0Ac",
  description: "\\r\\n in value",
});

formTest({
  name: "a",
  value: "b\n\rc",
  expected: "a=b%0D%0A%0D%0Ac",
  description: "\\n\\r in value",
});

formTest({
  name: "a",
  value: new File([], "b\nc"),
  expected: "a=b%0D%0Ac",
  description: "\\n in filename",
});

formTest({
  name: "a",
  value: new File([], "b\rc"),
  expected: "a=b%0D%0Ac",
  description: "\\r in filename",
});

formTest({
  name: "a",
  value: new File([], "b\r\nc"),
  expected: "a=b%0D%0Ac",
  description: "\\r\\n in filename",
});

formTest({
  name: "a",
  value: new File([], "b\n\rc"),
  expected: "a=b%0D%0A%0D%0Ac",
  description: "\\n\\r in filename",
});

formTest({
  name: 'a"b',
  value: "c",
  expected: "a%22b=c",
  description: "double quote in name",
});

formTest({
  name: "a",
  value: 'b"c',
  expected: "a=b%22c",
  description: "double quote in value",
});

formTest({
  name: "a",
  value: new File([], 'b"c'),
  expected: "a=b%22c",
  description: "double quote in filename",
});

formTest({
  name: "a'b",
  value: "c",
  expected: "a%27b=c",
  description: "single quote in name",
});

formTest({
  name: "a",
  value: "b'c",
  expected: "a=b%27c",
  description: "single quote in value",
});

formTest({
  name: "a",
  value: new File([], "b'c"),
  expected: "a=b%27c",
  description: "single quote in filename",
});

formTest({
  name: "a\\b",
  value: "c",
  expected: "a%5Cb=c",
  description: "backslash in name",
});

formTest({
  name: "a",
  value: "b\\c",
  expected: "a=b%5Cc",
  description: "backslash in value",
});

formTest({
  name: "a",
  value: new File([], "b\\c"),
  expected: "a=b%5Cc",
  description: "backslash in filename",
});

formTest({
  name: "áb",
  value: "ç",
  expected: "%C3%A1b=%C3%A7",
  description: "non-ASCII in name and value",
});

formTest({
  name: "a",
  value: new File([], "ə.txt"),
  expected: "a=%C9%99.txt",
  description: "non-ASCII in filename",
});

formTest({
  name: "aəb",
  value: "c\uFFFDd",
  formEncoding: "windows-1252",
  expected: "a%26%23601%3Bb=c%26%2365533%3Bd",
  description: "characters not in encoding in name and value",
});

formTest({
  name: "á",
  value: new File([], "💩"),
  formEncoding: "windows-1252",
  expected: "%E1=%26%23128169%3B",
  description: "character not in encoding in filename",
});

formTest({
  name: "\uD800",
  value: "\uD800",
  formEncoding: "windows-1252",
  expected: "%26%2365533%3B=%26%2365533%3B",
  description: "lone surrogate in name and value",
});
