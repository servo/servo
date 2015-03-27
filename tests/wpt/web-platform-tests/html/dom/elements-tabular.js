// Up-to-date as of 2013-04-12.
var tabularElements = {
  table: {
    // Conforming
    sortable: "boolean",

    // Obsolete
    align: "string",
    border: "string",
    frame: "string",
    rules: "string",
    summary: "string",
    width: "string",
    bgColor: {type: "string", treatNullAsEmptyString: true},
    cellPadding: {type: "string", treatNullAsEmptyString: true},
    cellSpacing: {type: "string", treatNullAsEmptyString: true},
  },
  caption: {
    // Obsolete
    align: "string",
  },
  colgroup: {
    span: "limited unsigned long",

    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
    width: "string",
  },
  col: {
    // Conforming
    span: "limited unsigned long",

    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
    width: "string",
  },
  tbody: {
    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
  },
  thead: {
    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
  },
  tfoot: {
    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
  },
  tr: {
    // Obsolete
    align: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    vAlign: "string",
    bgColor: {type: "string", treatNullAsEmptyString: true},
  },
  td: {
    // HTMLTableCellElement (Conforming)
    colSpan: {type: "unsigned long", defaultVal: 1},
    rowSpan: {type: "unsigned long", defaultVal: 1},
    headers: "settable tokenlist",

    // HTMLTableCellElement (Obsolete)
    align: "string",
    axis: "string",
    height: "string",
    width: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    noWrap: "boolean",
    vAlign: "string",
    bgColor: {type: "string", treatNullAsEmptyString: true},

    // HTMLTableDataCellElement (Obsolete)
    abbr: "string",
  },
  th: {
    // HTMLTableCellElement (Conforming)
    colSpan: {type: "unsigned long", defaultVal: 1},
    rowSpan: {type: "unsigned long", defaultVal: 1},
    headers: "settable tokenlist",

    // HTMLTableCellElement (Obsolete)
    align: "string",
    axis: "string",
    height: "string",
    width: "string",
    ch: {type: "string", domAttrName: "char"},
    chOff: {type: "string", domAttrName: "charoff"},
    noWrap: "boolean",
    vAlign: "string",
    bgColor: {type: "string", treatNullAsEmptyString: true},

    // HTMLTableHeaderCellElement (Conforming)
    // TODO: double-check that the way we're treating missing value
    // defaults is consistent here.  scope has an auto state with no
    // associated keyword, which is the missing value default -- is this
    // the right syntax for that?
    scope: {type: "enum", keywords: ["row", "col", "rowgroup", "colgroup"]},
    abbr: "string",
    sorted: "string",
  },
};

mergeElements(tabularElements);
