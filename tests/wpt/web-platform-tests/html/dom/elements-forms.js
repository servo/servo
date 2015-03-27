// Up-to-date as of 2013-04-07.
var formElements = {
  form: {
    acceptCharset: {type: "string", domAttrName: "accept-charset"},
    // TODO: action is special
    // action: "url",
    autocomplete: {type: "enum", keywords: ["on", "off"], defaultVal: "on"},
    enctype: {type: "enum", keywords: ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], defaultVal: "application/x-www-form-urlencoded"},
    encoding: {type: "enum", keywords: ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], defaultVal: "application/x-www-form-urlencoded", domAttrName: "enctype"},
    method: {type: "enum", keywords: ["get", "post", "dialog"], defaultVal: "get"},
    name: "string",
    noValidate: "boolean",
    target: "string",
  },
  fieldset: {
    disabled: "boolean",
    name: "string",
  },
  legend: {
    // Obsolete
    align: "string",
  },
  label: {
    htmlFor: {type: "string", domAttrName: "for"},
  },
  input: {
    // Conforming
    accept: "string",
    alt: "string",
    // TODO: autocomplete is special.
    // autocomplete: {type: "enum", keywords: ["on", "off"], defaultVal: "on"},
    autofocus: "boolean",
    defaultChecked: {type: "boolean", domAttrName: "checked"},
    dirName: "string",
    disabled: "boolean",
    // TODO: formAction is special
    // formAction: "url",
    formEnctype: {type: "enum", keywords: ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], invalidVal: "application/x-www-form-urlencoded"},
    formMethod: {type: "enum", keywords: ["get", "post"], invalidVal: "get"},
    formNoValidate: "boolean",
    formTarget: "string",
    //TODO: only reflected on setting
    //height: "unsigned long",
    inputMode: {type: "enum", keywords: ["verbatim", "latin", "latin-name", "latin-prose", "full-width-latin", "kana", "katakana", "numeric", "tel", "email", "url"]},
    max: "string",
    maxLength: "limited long",
    min: "string",
    multiple: "boolean",
    name: "string",
    pattern: "string",
    placeholder: "string",
    readOnly: "boolean",
    required: "boolean",
    // https://html.spec.whatwg.org/multipage/#attr-input-size
    size: {type: "limited unsigned long", defaultVal: 20},
    src: "url",
    step: "string",
    type: {type: "enum", keywords: ["hidden", "text", "search", "tel",
      "url", "email", "password", "datetime", "date", "month", "week",
      "time", "datetime-local", "number", "range", "color", "checkbox",
      "radio", "file", "submit", "image", "reset", "button"], defaultVal:
      "text"},
    //TODO: only reflected on setting
    //width: "unsigned long",
    defaultValue: {type: "string", domAttrName: "value"},

    // Obsolete
    align: "string",
    useMap: "string",
  },
  button: {
    autofocus: "boolean",
    disabled: "boolean",
    // TODO: formAction is special
    // formAction: "url",
    formEnctype: {type: "enum", keywords: ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], invalidVal: "application/x-www-form-urlencoded"},
    formMethod: {type: "enum", keywords: ["get", "post", "dialog"], invalidVal: "get"},
    formNoValidate: "boolean",
    formTarget: "string",
    name: "string",
    type: {type: "enum", keywords: ["submit", "reset", "button"], defaultVal: "submit"},
    value: "string",
    // TODO: menu
  },
  select: {
    autofocus: "boolean",
    disabled: "boolean",
    multiple: "boolean",
    name: "string",
    required: "boolean",
    size: {type: "unsigned long", defaultVal: 0},
  },
  datalist: {},
  optgroup: {
    disabled: "boolean",
    label: "string",
  },
  option: {
    disabled: "boolean",
    label: {type: "string", customGetter: true},
    defaultSelected: {type: "boolean", domAttrName: "selected"},
    value: {type: "string", customGetter: true},
  },
  textarea: {
    // TODO: autocomplete is special.
    // autocomplete: {type: "enum", keywords: ["on", "off"], defaultVal: "on"},
    autofocus: "boolean",
    cols: {type: "limited unsigned long", defaultVal: 20},
    dirName: "string",
    disabled: "boolean",
    inputMode: {type: "enum", keywords: ["verbatim", "latin", "latin-name", "latin-prose", "full-width-latin", "kana", "katakana", "numeric", "tel", "email", "url"]},
    maxLength: "limited long",
    name: "string",
    placeholder: "string",
    readOnly: "boolean",
    required: "boolean",
    rows: {type: "limited unsigned long", defaultVal: 2},
    wrap: "string",
  },
  keygen: {
    autofocus: "boolean",
    challenge: "string",
    disabled: "boolean",
    // The invalid value default is the "unknown" state, which for our
    // purposes  seems to be the same as having no invalid value default.
    // The missing  value default depends on whether "rsa" is implemented,
    // so we use null,  which is magically reserved for "don't try testing
    // this", since no one  default is required.  (TODO: we could test that
    // it's either the RSA  state or the unknown state.)
    keytype: {type: "enum", keywords: ["rsa"], defaultVal: null},
    name: "string",
  },
  output: {
    htmlFor: {type: "settable tokenlist", domAttrName: "for" },
    name: "string",
  },
  progress: {
    max: {type: "limited double", defaultVal: 1.0},
  },
  meter: {},
};

mergeElements(formElements);
