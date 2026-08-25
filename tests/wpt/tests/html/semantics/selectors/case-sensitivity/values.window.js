// https://html.spec.whatwg.org/#case-sensitivity-of-selectors
[
  "accept",
  "accept-charset",
  "align",
  "alink",
  "axis",
  "bgcolor",
  "charset",
  "checked",
  "clear",
  "codetype",
  "color",
  "compact",
  "declare",
  "defer",
  "dir",
  "direction",
  "disabled",
  "enctype",
  "face",
  "frame",
  "hreflang",
  "http-equiv",
  "lang",
  "language",
  "link",
  "media",
  "method",
  "multiple",
  "nohref",
  "noresize",
  "noshade",
  "nowrap",
  "readonly",
  "rel",
  "rev",
  "rules",
  "scope",
  "scrolling",
  "selected",
  "shape",
  "target",
  "text",
  "type",
  "valign",
  "valuetype",
  "vlink",
].forEach(attributeName => {
  const xmlDocument = new Document();
  const htmlDocument = document;
  [
    {
      input: xmlDocument.createElementNS("http://www.w3.org/1999/xhtml", "a"),
      expected: false,
      title: "<html:a> in XML",
    },
    {
      input: xmlDocument.createElementNS("http://www.w3.org/1999/xhtml", "unknown"),
      expected: false,
      title: "<html:unknown> in XML",
    },
    {
      input: xmlDocument.createElementNS("", "unknown"),
      expected: false,
      title: "<:unknown> in XML"
    },
    {
      input: htmlDocument.createElementNS("http://www.w3.org/1999/xhtml", "a"),
      expected: true,
      title: "<html:a> in HTML",
    },
    {
      input: htmlDocument.createElementNS("http://www.w3.org/1999/xhtml", "unknown"),
      expected: true,
      title: "<html:unknown> in HTML",
    },
    {
      input: htmlDocument.createElementNS("", "unknown"),
      expected: false,
      title: "<:unknown> in HTML"
    },
  ].forEach(({ input, expected, title }) => {
    test(t => {
      t.add_cleanup(() => input.removeAttribute(attributeName));
      input.setAttribute(attributeName, "HEYÏ");
      assert_equals(input.matches(`[${attributeName}^=hey]`), expected, `^=hey`);
      assert_false(input.matches(`[${attributeName}^=heyi]`));
    }, `${attributeName}'s value is properly ASCII-case-insensitive for ${title}`);
  });
});
