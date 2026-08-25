var divElement = {
  div: {
    ariaAtomic: {type: "enum", domAttrName: "aria-atomic", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: null},
    ariaAutoComplete: {type: "enum", domAttrName: "aria-autocomplete", keywords: ["inline", "list", "both", "none"], isNullable: true, invalidVal: "none", defaultVal: "none"},
    ariaBusy: {type: "enum", domAttrName: "aria-busy", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaChecked: {type: "enum", domAttrName: "aria-checked", keywords: ["true", "false", "mixed"], nonCanon: {"": null}, isNullable: true, invalidVal: null, defaultVal: null},
    ariaCurrent: {type: "enum", domAttrName: "aria-current", keywords: ["page", "step", "location", "date", "time", "true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "true", defaultVal: "false"},
    ariaDisabled: {type: "enum", domAttrName: "aria-disabled", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaExpanded: {type: "enum", domAttrName: "aria-expanded", keywords: ["true", "false"], nonCanon: {"": null}, isNullable: true, invalidVal: null, defaultVal: null},
    ariaHasPopup: {type: "enum", domAttrName: "aria-haspopup", keywords: ["true", "false", "menu", "dialog", "listbox", "tree", "grid"], isNullable: true, invalidVal: "false", defaultVal: null},
    ariaHidden: {type: "enum", domAttrName: "aria-hidden", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaInvalid: {type: "enum", domAttrName: "aria-invalid", keywords: ["true", "false", "spelling", "grammar"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "true", defaultVal: "false"},
    ariaLive: {type: "enum", domAttrName: "aria-live", keywords: ["polite", "assertive", "off"], isNullable: true, invalidVal: "off", defaultVal: "off"},
    ariaModal: {type: "enum", domAttrName: "aria-modal", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaMultiLine: {type: "enum", domAttrName: "aria-multiline", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaMultiSelectable: {type: "enum", domAttrName: "aria-multiselectable", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaOrientation: {type: "enum", domAttrName: "aria-orientation", keywords: ["horizontal", "vertical"], nonCanon: {"": null}, isNullable: true, invalidVal: null, defaultVal: null},
    ariaPressed: {type: "enum", domAttrName: "aria-pressed", keywords: ["true", "false", "mixed"], nonCanon: {"": null}, isNullable: true, invalidVal: null, defaultVal: null},
    ariaReadOnly: {type: "enum", domAttrName: "aria-readonly", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaRequired: {type: "enum", domAttrName: "aria-required", keywords: ["true", "false"], nonCanon: {"": "false"}, isNullable: true, invalidVal: "false", defaultVal: "false"},
    ariaSelected: {type: "enum", domAttrName: "aria-selected", keywords: ["true", "false"], nonCanon: {"": null}, isNullable: true, invalidVal: null, defaultVal: null},
    ariaSort: {type: "enum", domAttrName: "aria-sort", keywords: ["ascending", "descending", "other", "none"], isNullable: true, invalidVal: "none", defaultVal: "none"}
  },
};

mergeElements(divElement);
