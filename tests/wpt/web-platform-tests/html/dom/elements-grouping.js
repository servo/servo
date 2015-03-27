// Up-to-date as of 2013-04-08.
var groupingElements = {
  p: {
    // Obsolete
    align: "string",
  },
  hr: {
    // Obsolete
    align: "string",
    color: "string",
    noShade: "boolean",
    size: "string",
    width: "string",
  },
  pre: {
    // Obsolete
    width: "long",
  },
  blockquote: {
    cite: "url",
  },
  ol: {
    // Conforming
    reversed: "boolean",
    // TODO: This should have a default value of the list's length if the
    // reversed attribute is set.
    start: {type: "long", defaultVal: 1},
    type: "string",

    // Obsolete
    compact: "boolean",
  },
  ul: {
    // Obsolete
    compact: "boolean",
    type: "string",
  },
  li: {
    // Conforming
    value: "long",

    // Obsolete
    type: "string",
  },
  dl: {
    // Obsolete
    compact: "boolean",
  },
  dt: {},
  dd: {},
  figure: {},
  figcaption: {},
  main: {},
  div: {
    // Obsolete
    align: "string",
  },
};

mergeElements(groupingElements);
