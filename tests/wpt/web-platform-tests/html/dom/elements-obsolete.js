// Up-to-date as of 2013-04-13.
var obsoleteElements = {
  // https://html.spec.whatwg.org/multipage/#the-applet-element
  applet: {
    align: "string",
    alt: "string",
    archive: "string",
    code: "string",
    codeBase: "url",
    height: "string",
    hspace: "unsigned long",
    name: "string",
    object: "url",
    vspace: "unsigned long",
    width: "string",
  },
  // https://html.spec.whatwg.org/multipage/#the-marquee-element-2
  marquee: {
    behavior: "string",
    bgColor: "string",
    direction: "string",
    height: "string",
    hspace: "unsigned long",
    scrollAmount: {type: "unsigned long", defaultVal: 6},
    scrollDelay: {type: "unsigned long", defaultVal: 85},
    trueSpeed: "boolean",
    vspace: "unsigned long",
    width: "string",
  },
  // https://html.spec.whatwg.org/multipage/#frameset
  frameset: {
    cols: "string",
    rows: "string",
  },
  // https://html.spec.whatwg.org/multipage/#frame
  frame: {
    name: "string",
    scrolling: "string",
    src: "url",
    frameBorder: "string",
    longDesc: "url",
    noResize: "boolean",
    marginHeight: {type: "string", treatNullAsEmptyString: true},
    marginWidth: {type: "string", treatNullAsEmptyString: true},
  },
  // https://html.spec.whatwg.org/multipage/#htmldirectoryelement
  dir: {
    compact: "boolean",
  },
  // https://html.spec.whatwg.org/multipage/#htmlfontelement
  font: {
    color: {type: "string", treatNullAsEmptyString: true},
    face: "string",
    size: "string",
  },
};

mergeElements(obsoleteElements);
