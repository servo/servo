// Up-to-date as of 2013-04-06.
var embeddedElements = {
  img: {
    // Conforming
    alt: "string",
    src: "url",
    srcset: "string",
    crossOrigin: {type: "enum", keywords: ["anonymous", "use-credentials"], nonCanon:{"": "anonymous"}, isNullable: true, defaultVal: null, invalidVal: "anonymous"},
    useMap: "string",
    isMap: "boolean",
    width: {type: "unsigned long", customGetter: true},
    height: {type: "unsigned long", customGetter: true},

    // Obsolete
    name: "string",
    lowsrc: {type: "url"},
    align: "string",
    hspace: "unsigned long",
    vspace: "unsigned long",
    longDesc: "url",
    border: {type: "string", treatNullAsEmptyString: true},
  },
  iframe: {
    // Conforming
    src: "url",
    srcdoc: "string",
    name: "string",
    sandbox: "settable tokenlist",
    seamless: "boolean",
    allowFullscreen: "boolean",
    width: "string",
    height: "string",

    // Obsolete
    align: "string",
    scrolling: "string",
    frameBorder: "string",
    longDesc: "url",
    marginHeight: {type: "string", treatNullAsEmptyString: true},
    marginWidth: {type: "string", treatNullAsEmptyString: true}
  },
  embed: {
    // Conforming
    src: "url",
    type: "string",
    width: "string",
    height: "string",

    // Obsolete
    align: "string",
    name: "string"
  },
  object: {
    // Conforming
    data: "url",
    type: "string",
    typeMustMatch: "boolean",
    name: "string",
    useMap: "string",
    width: "string",
    height: "string",

    // Obsolete
    align: "string",
    archive: "string",
    code: "string",
    declare: "boolean",
    hspace: "unsigned long",
    standby: "string",
    vspace: "unsigned long",
    codeBase: "url",
    codeType: "string",
    border: {type: "string", treatNullAsEmptyString: true}
  },
  param: {
    // Conforming
    name: "string",
    value: "string",

    // Obsolete
    type: "string",
    valueType: "string"
  },
  video: {
    // HTMLMediaElement
    src: "url",
    crossOrigin: {type: "enum", keywords: ["anonymous", "use-credentials"], nonCanon:{"": "anonymous"}, isNullable: true, defaultVal: null, invalidVal: "anonymous"},
    // As with "keytype", we have no missing value default defined here.
    preload: {type: "enum", keywords: ["none", "metadata", "auto"], nonCanon: {"": "auto"}, defaultVal: null},
    autoplay: "boolean",
    loop: "boolean",
    mediaGroup: "string",
    controls: "boolean",
    defaultMuted: {type: "boolean", domAttrName: "muted"},

    width: "unsigned long",
    height: "unsigned long",
    poster: "url"
  },
  audio: {
    // HTMLMediaElement
    src: "url",
    crossOrigin: {type: "enum", keywords: ["anonymous", "use-credentials"], nonCanon:{"": "anonymous"}, isNullable: true, defaultVal: null, invalidVal: "anonymous"},
    // As with "keytype", we have no missing value default defined here.
    preload: {type: "enum", keywords: ["none", "metadata", "auto"], nonCanon: {"": "auto"}, defaultVal: null},
    autoplay: "boolean",
    loop: "boolean",
    mediaGroup: "string",
    controls: "boolean",
    defaultMuted: {type: "boolean", domAttrName: "muted"}
  },
  source: {
    src: "url",
    type: "string",
    media: "string"
  },
  track: {
    kind: {type: "enum", keywords: ["subtitles", "captions", "descriptions", "chapters", "metadata"], defaultVal: "subtitles", invalidVal: "metadata"},
    src: "url",
    srclang: "string",
    label: "string",
    "default": "boolean"
  },
  canvas: {
    width: {type: "unsigned long", defaultVal: 300},
    height: {type: "unsigned long", defaultVal: 150}
  },
  map: {
    name: "string"
  },
  area: {
    // Conforming
    alt: "string",
    coords: "string",
    shape: "string",
    target: "string",
    download: "string",
    ping: "urls",
    rel: "string",
    relList: {type: "tokenlist", domAttrName: "rel"},
    hreflang: "string",
    type: "string",

    // HTMLHyperlinkElementUtils
    href: "url",

    // Obsolete
    noHref: "boolean"
  },
};

mergeElements(embeddedElements);
