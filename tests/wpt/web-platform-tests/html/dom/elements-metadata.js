// Up-to-date as of 2013-04-08.
var metadataElements = {
  head: {},
  title: {},
  base: {
    // XXX href is weird. href: "url",
    target: "string",
  },
  link: {
    // Conforming
    href: "url",
    crossOrigin: {type: "enum", keywords: ["anonymous", "use-credentials"], nonCanon:{"": "anonymous"}, isNullable: true, defaultVal: null, invalidVal: "anonymous"},
    rel: "string",
    relList: {type: "tokenlist", domAttrName: "rel"},
    // as: {}, XXX TODO: reflecting IDL attribute is an IDL enumeration
    media: "string",
    nonce: "string",
    integrity: "string",
    hreflang: "string",
    type: "string",
    sizes: "settable tokenlist",
    referrerPolicy: {type: "enum", keywords: ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"]},

    // Obsolete
    charset: "string",
    rev: "string",
    target: "string",
  },
  meta: {
    // Conforming
    name: "string",
    httpEquiv: {type: "string", domAttrName: "http-equiv"},
    content: "string",

    // Obsolete
    scheme: "string",
  },
  style: {
    media: "string",
    type: "string",
  },
};

mergeElements(metadataElements);
