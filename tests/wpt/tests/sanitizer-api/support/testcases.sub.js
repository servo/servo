const testcases = [
  { config_input: {}, value: "test", result: "test", message: "string" },
  {
    config_input: {},
    value: "<b>bla</b>",
    result: "<b>bla</b>",
    message: "html fragment",
  },
  { config_input: {}, value: "<a<embla", result: "", message: "broken html" },
  {
    config_input: {},
    value: {},
    result: "[object Object]",
    message: "empty object",
  },
  { config_input: {}, value: 1, result: "1", message: "number" },
  { config_input: {}, value: 000, result: "0", message: "zeros" },
  { config_input: {}, value: 1 + 2, result: "3", message: "arithmetic" },
  { config_input: {}, value: "", result: "", message: "empty string" },
  {
    config_input: {},
    value: undefined,
    result: "undefined",
    message: "undefined",
  },
  {
    config_input: {},
    value: "<html><head></head><body>test</body></html>",
    result: "test",
    message: "document",
  },
  {
    config_input: {},
    value: "<div>test",
    result: "<div>test</div>",
    message: "html without close tag",
  },
  {
    config_input: {},
    value: "<script>alert('i am a test')</script>",
    result: "",
    message: "scripts for default configs",
  },
  {
    config_input: {},
    value: "hello<script>alert('i am a test')</script>",
    result: "hello",
    message: "script not as root",
  },
  {
    config_input: {},
    value: "<div><b>hello<script>alert('i am a test')</script>",
    result: "<div><b>hello</b></div>",
    message: "script deeper in the tree",
  },
  {
    config_input: {},
    value: "<p onclick='a= 123'>Click.</p>",
    result: "<p>Click.</p>",
    message: "onclick scripts",
  },
  {
    config_input: {},
    value: "<plaintext><p>text</p>",
    result: "&lt;p&gt;text&lt;/p&gt;",
    message: "plaintext",
  },
  {
    config_input: {},
    value: "<xmp>TEXT</xmp>",
    result: "TEXT",
    message: "xmp",
  },
  {
    config_input: { test: 123 },
    value: "test",
    result: "test",
    message: "invalid config_input",
  },
  {
    config_input: { removeElements: [] },
    value: "test",
    result: "test",
    message: "empty removeElements list",
  },
  {
    config_input: { removeElements: ["div"] },
    value: "<div>test</div><p>bla",
    result: "<p>bla</p>",
    message: "test html without close tag with removeElements list ['div']",
  },
  {
    config_input: {},
    value: "<custom-element>test</custom-element>bla",
    result: "bla",
    message: "default behavior for custom elements",
  },
  {
    config_input: { customElements: true },
    value: "<custom-element>test</custom-element>bla",
    result: "testbla",
    message: "allow custom elements",
  },
  {
    config_input: {
      customElements: true,
      elements: ["custom-element"],
    },
    value: "<custom-element>test</custom-element>bla",
    result: "<custom-element>test</custom-element>bla",
    message: "allow custom elements with allow elements",
  },
  {
    config_input: { customElements: false },
    value: "<custom-element>test</custom-element>bla",
    result: "bla",
    message: "disallow custom elements",
  },
  {
    config_input: {
      removeElements: ["custom-element"],
      customElements: true,
    },
    value: "<custom-element>test</custom-element>bla",
    result: "bla",
    message: 'allow custom elements with drop list contains ["custom-element"]',
  },
  {
    config_input: { removeElements: ["script"] },
    value: "<script>alert('i am a test')</script>",
    result: "",
    message: 'test script with ["script"] as removeElements list',
  },
  {
    config_input: { removeElements: ["test-element", "i"] },
    value: "<div>balabala<i>test</i></div><test-element>t</test-element>",
    result: "<div>balabala</div>",
    message: 'removeElements list ["test-element", "i"]}',
  },
  {
    config_input: { removeElements: ["dl", "p"] },
    value: "<div>balabala<i>i</i><p>t</p></div>",
    result: "<div>balabala<i>i</i></div>",
    message: 'removeElements list ["dl", "p"]}',
  },
  {
    config_input: { elements: ["p"] },
    value: "<div>test<div>p</div>tt<p>div</p></div>",
    result: "testptt<p>div</p>",
    message: 'elements list ["p"]',
  },
  {
    config_input: { removeElements: ["div"], elements: ["div"] },
    value: "<div>test</div><p>bla",
    result: "bla",
    message: "elements list has no influence to removeElements",
  },
  {
    config_input: { removeAttributes: [] },
    value: "<p id='test'>Click.</p>",
    result: '<p id="test">Click.</p>',
    message: "empty removeAttributes list with id attribute",
  },
  {
    config_input: { removeAttributes: ["id"] },
    value: "<p id='test'>Click.</p>",
    result: "<p>Click.</p>",
    message: 'removeAttributes list ["id"] with id attribute',
  },
  {
    config_input: {
      removeAttributes: ["data-attribute-with-dashes"],
    },
    value:
      "<p id='p' data-attribute-with-dashes='123'>Click.</p><script>document.getElementById('p').dataset.attributeWithDashes=123;</script>",
    result: '<p id="p">Click.</p>',
    message:
      'removeAttributes list ["data-attribute-with-dashes"] with dom dataset js access',
  },
  {
    config_input: {
      elements: [
        { name: "p", attributes: ["title"] },
        { name: "div", attributes: ["id"] },
      ],
    },
    value: "<p id='p' title='p'>P</p><div id='div' title='div'>DIV</div>",
    result: '<p title="p">P</p><div id="div">DIV</div>',
    message:
      'elements list with <p> attributes: ["title"] and div attributes: ["id"] lists',
  },
  {
    config_input: {
      elements: [
        { name: "p", removeAttributes: ["title"] },
        { name: "div", removeAttributes: ["id"] },
      ],
    },
    value: "<p id='p' title='p'>P</p><div id='div' title='div'>DIV</div>",
    result: '<p id="p">P</p><div title="div">DIV</div>',
    message:
      'elements list with <p> removeAttributes: ["title"]  and div removeAttributes: ["id"] lists',
  },
  {
    config_input: {
      elements: [{ name: "div", attributes: ["id"], removeAttributes: ["id"] }],
    },
    value: "<div id='div' title='div'>DIV</div>",
    result: "<div>DIV</div>",
    message:
      'elements list with <div> attributes: ["id"] and removeAttributes: ["id"] lists',
  },
  {
    config_input: {
      elements: [{ name: "div", attributes: ["id", "title"] }],
      attributes: []
    },
    value: "<div id='div' title='div'>DIV</div>",
    result: "<div>DIV</div>",
    message:
      'elements list with <div> attributes: ["id", "title"] does not override empty attributes: [] list',
  },
  {
    config_input: {
      elements: [{ name: "div", attributes: ["id", "title"] }],
      removeAttributes: ["id", "title"]
    },
    value: "<div id='div' title='div'>DIV</div>",
    result: "<div>DIV</div>",
    message:
      'elements list with <div> attributes: ["id", "title"] does not override removeAttributes: ["id", "title"] list',
  },
  {
    config_input: {
      elements: [{ name: "div", removeAttributes: ["id", "title"] }],
      attributes: ["id", "title"]
    },
    value: "<div id='div' title='div'>DIV</div>",
    result: "<div>DIV</div>",
    message:
      'elements list with <div> removeAttributes: ["id", "title"] is effective even with attributes: ["id", "title"] list',
  },
  {
    config_input: { attributes: ["id"] },
    value: "<p id='test' onclick='a= 123'>Click.</p>",
    result: '<p id="test">Click.</p>',
    message: 'attributes list ["id"] with id attribute and onclick scripts',
  },
  // {config_input: {allowAttributes: {"*": ["a"]}}, value: "<a id='a' style='color: black'>Click.</a><div style='color: white'>div</div>", result: "<a id=\"a\" style=\"color: black\">Click.</a><div>div</div>", message: "allowAttributes list {\"*\": [\"a\"]} with style attribute"},
  {
    config_input: {
      removeAttributes: ["style"],
      attributes: ["style"],
    },
    value: "<p style='color: black'>Click.</p>",
    result: "<p>Click.</p>",
    message: "attributes list has no influence to removeAttributes list",
  },
  {
    config_input: { elements: ["template", "div"] },
    value: "<template><script>test</script><div>hello</div></template>",
    result: "<template><div>hello</div></template>",
    message: "Template element",
  },
  {
    config_input: {},
    value: "<a href='javascript:evil.com'>Click.</a>",
    result: "<a>Click.</a>",
    message: "HTMLAnchorElement with javascript protocal",
  },
  {
    config_input: {},
    value: "<a href='  javascript:evil.com'>Click.</a>",
    result: "<a>Click.</a>",
    message: "HTMLAnchorElement with javascript protocal start with space",
  },
  {
    config_input: {},
    value: "<a href='http:evil.com'>Click.</a>",
    result: '<a href="http:evil.com">Click.</a>',
    message: "HTMLAnchorElement",
  },
  {
    config_input: {},
    value: "<area href='javascript:evil.com'>Click.</area>",
    result: "<area>Click.",
    message: "HTMLAreaElement with javascript protocal",
  },
  {
    config_input: {},
    value: "<area href=' javascript:evil.com'>Click.</area>",
    result: "<area>Click.",
    message: "HTMLAreaElement with javascript protocal start with space",
  },
  {
    config_input: {},
    value: "<area href='http:evil.com'>Click.</area>",
    result: '<area href="http:evil.com">Click.',
    message: "HTMLAreaElement",
  },
  {
    config_input: {},
    value: "<form action='javascript:evil.com'>Click.</form>",
    result: "<form>Click.</form>",
    message: "HTMLFormElement with javascript action",
  },
  {
    config_input: {},
    value: "<form action=' javascript:evil.com'>Click.</form>",
    result: "<form>Click.</form>",
    message: "HTMLFormElement with javascript action start with space",
  },
  {
    config_input: {},
    value: "<form action='http:evil.com'>Click.</form>",
    result: '<form action="http:evil.com">Click.</form>',
    message: "HTMLFormElement",
  },
  {
    config_input: {},
    value: "<input formaction='javascript:evil.com'>Click.</input>",
    result: "<input>Click.",
    message: "HTMLInputElement with javascript formaction",
  },
  {
    config_input: {},
    value: "<input formaction=' javascript:evil.com'>Click.</input>",
    result: "<input>Click.",
    message: "HTMLInputElement with javascript formaction start with space",
  },
  {
    config_input: {},
    value: "<input formaction='http:evil.com'>Click.</input>",
    result: '<input formaction="http:evil.com">Click.',
    message: "HTMLInputElement",
  },
  {
    config_input: {},
    value: "<button formaction='javascript:evil.com'>Click.</button>",
    result: "<button>Click.</button>",
    message: "HTMLButtonElement with javascript formaction",
  },
  {
    config_input: {},
    value: "<button formaction=' javascript:evil.com'>Click.</button>",
    result: "<button>Click.</button>",
    message: "HTMLButtonElement with javascript formaction start with space",
  },
  {
    config_input: {},
    value: "<button formaction='http:evil.com'>Click.</button>",
    result: '<button formaction="http:evil.com">Click.</button>',
    message: "HTMLButtonElement",
  },
  {
    config_input: {},
    value:
      "<p>Some text</p></body><!-- 1 --></html><!-- 2 --><p>Some more text</p>",
    result: "<p>Some text</p><p>Some more text</p>",
    message: "malformed HTML",
  },
  {
    config_input: {},
    value: "<p>Some text</p><!-- 1 --><!-- 2 --><p>Some more text</p>",
    result: "<p>Some text</p><p>Some more text</p>",
    message: "HTML with comments; comments not allowed",
  },
  {
    config_input: { comments: true },
    value: "<p>Some text</p><!-- 1 --><!-- 2 --><p>Some more text</p>",
    result: "<p>Some text</p><!-- 1 --><!-- 2 --><p>Some more text</p>",
    message: "HTML with comments; comments",
  },
  {
    config_input: { comments: false },
    value: "<p>Some text</p><!-- 1 --><!-- 2 --><p>Some more text</p>",
    result: "<p>Some text</p><p>Some more text</p>",
    message: "HTML with comments; !comments",
  },
  {
    config_input: {},
    value: "<p>comment<!-- hello -->in<!-- </p> -->text</p>",
    result: "<p>commentintext</p>",
    message: "HTML with comments deeper in the tree",
  },
  {
    config_input: { comments: true },
    value: "<p>comment<!-- hello -->in<!-- </p> -->text</p>",
    result: "<p>comment<!-- hello -->in<!-- </p> -->text</p>",
    message: "HTML with comments deeper in the tree, comments",
  },
  {
    config_input: { comments: false },
    value: "<p>comment<!-- hello -->in<!-- </p> -->text</p>",
    result: "<p>commentintext</p>",
    message: "HTML with comments deeper in the tree, !comments",
  },
  {
    config_input: { elements: ["svg"] },
    value: "<svg></svg>",
    result: "",
    message:
      "Unknown HTML names (HTMLUnknownElement instances) should not match elements parsed as non-HTML namespaces.",
  },
  {
    config_input: { elements: ["div", "svg"] },
    value: "<div><svg></svg></div>",
    result: "<div></div>",
    message:
      "Unknown HTML names (HTMLUnknownElement instances) should not match elements parsed as non-HTML namespaces when nested.",
  },

  // Case normalization (actually: lack of)
  {
    config_input: { removeElements: ["I", "DL"] },
    value: "<div>balabala<dl>test</dl></div>",
    result: "<div>balabala<dl>test</dl></div>",
    message: 'removeElements list ["I", "DL"]}',
  },
  {
    config_input: { removeElements: ["i", "dl"] },
    value: "<div>balabala<dl>test</dl></div>",
    result: "<div>balabala</div>",
    message: 'removeElements list ["i", "dl"]}',
  },
  {
    config_input: { removeElements: ["i", "dl"] },
    value: "<DIV>balabala<DL>test</DL></DIV>",
    result: "<div>balabala</div>",
    message: 'removeElements list ["i", "dl"]} with uppercase HTML',
  },
  {
    config_input: { removeAttributes: ["ID"] },
    value: '<p id="test">Click.</p>',
    result: '<p id="test">Click.</p>',
    message: 'removeAttributes list ["ID"] with id attribute',
  },
  {
    config_input: { removeAttributes: ["ID"] },
    value: '<p ID="test">Click.</p>',
    result: '<p id="test">Click.</p>',
    message: 'removeAttributes list ["ID"] with ID attribute',
  },
  {
    config_input: { removeAttributes: ["id"] },
    value: '<p ID="test">Click.</p>',
    result: "<p>Click.</p>",
    message: 'removeAttributes list ["id"] with ID attribute',
  },

  // unknownMarkup for elements (with and without)
  {
    config_input: { removeElements: [123, "test", "i", "custom-element"] },
    value:
      "<div>balabala<i>test</i></div><test>t</test><custom-element>custom-element</custom-element>",
    result: "<div>balabala</div>",
    message: "removeElements with unknown elements and without unknownMarkup",
  },
  {
    config_input: {
      replaceWithChildrenElements: [123, "test", "i", "custom-element"],
    },
    value:
      "<div>balabala<i>test</i></div><test>t</test><custom-element>custom-element</custom-element>",
    result: "<div>balabalatest</div>",
    message:
      "replaceWithChildrenElements with unknown elements and without unknownMarkup",
  },
  {
    config_input: { elements: ["p", "test"] },
    value: "<div>test<div>p</div>tt<p>div</p></div><test>test</test>",
    result: "testptt<p>div</p>",
    message: "elements with unknown elements and without unknownMarkup",
  },
  {
    config_input: {
      removeElements: [123, "test", "i", "custom-element"],
      unknownMarkup: true,
    },
    value:
      "<div>balabala<i>test</i></div><test>t</test><custom-element>custom-element</custom-element>",
    result: "<div>balabala</div>",
    message: "removeElements with unknown elements and with unknownMarkup",
  },
  {
    config_input: {
      replaceWithChildrenElements: [123, "test", "i", "custom-element"],
      unknownMarkup: true,
    },
    value:
      "<div>balabala<i>test</i></div><test>t</test><custom-element>custom-element</custom-element>",
    result: "<div>balabalatest</div>t",
    message:
      "replaceWithChildrenElements with unknown elements and with unknownMarkup",
  },
  {
    config_input: { elements: ["p", "test"], unknownMarkup: true },
    value: "<div>test<div>p</div>tt<p>div</p><test>test</test></div>",
    result: "testptt<p>div</p><test>test</test>",
    message: "elements with unknown elements and with unknownMarkup",
  },

  // unknownMarkup for attributes (with and without)
  {
    config_input: {
      attributes: ["hello", "world"],
    },
    value: "<div hello='1' world='2'><b hello='3' world='4'>",
    result: "<div><b></b></div>",
    message: "attributes: unknown attributes and without unknownMarkup",
  },
  {
    config_input: {
      attributes: ["hello", "world"],
      unknownMarkup: true,
    },
    value: "<div hello='1' world='2'><b hello='3' world='4'>",
    result: '<div hello="1" world="2"><b hello="3" world="4"></b></div>',
    message: "attributes: unknown attributes and with unknownMarkup",
  },
  {
    config_input: {
      removeAttributes: ["hello", "world"],
    },
    value: "<div hello='1' world='2'><b hello='3' world='4'>",
    result: "<div><b></b></div>",
    message: "removeAttributes: unknown attributes and without unknownMarkup",
  },
  {
    config_input: {
      removeAttributes: ["hello", "world"],
      unknownMarkup: true,
    },
    value: "<div hello='1' world='2'><b hello='3' world='4'>",
    result: "<div><b></b></div>",
    message: "removeAttributes unknown attributes and with allowUnknownMarkup",
  },
];
