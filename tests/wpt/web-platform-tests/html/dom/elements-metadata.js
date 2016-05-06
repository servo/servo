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
    media: "string",
    hreflang: "string",
    type: "string",
    sizes: "settable tokenlist",

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
    scoped: "boolean",
  },
};

mergeElements(metadataElements);
