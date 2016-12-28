// Up-to-date as of 2013-04-19.
var textElements = {
  a: {
    // Conforming
    target: "string",
    download: "string",
    ping: "string",
    rel: "string",
    relList: {type: "tokenlist", domAttrName: "rel"},
    hreflang: "string",
    type: "string",
    referrerPolicy: {type: "enum", keywords: ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"]},

    // HTMLHyperlinkElementUtils
    href: "url",

    // Obsolete
    coords: "string",
    charset: "string",
    name: "string",
    rev: "string",
    shape: "string",
  },
  em: {},
  strong: {},
  small: {},
  s: {},
  cite: {},
  q: {
    cite: "url",
  },
  dfn: {},
  abbr: {},
  data: {
    value: "string",
  },
  time: {
    dateTime: "string",
  },
  code: {},
  // Opera 11.50 doesn't allow unquoted "var" here, although ES5 does and
  // other browsers support it.
  "var": {},
  samp: {},
  kbd: {},
  sub: {},
  sup: {},
  i: {},
  b: {},
  u: {},
  mark: {},
  ruby: {},
  rt: {},
  rp: {},
  bdi: {},
  bdo: {},
  span: {},
  br: {
    // Obsolete
    clear: "string",
  },
  wbr: {},
};

mergeElements(textElements);
