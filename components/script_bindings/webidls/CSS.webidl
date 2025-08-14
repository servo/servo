/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://drafts.csswg.org/cssom/#namespacedef-css
 */

// https://drafts.csswg.org/cssom/#the-css.escape()-method
[Exposed=Window]
namespace CSS {
  [Throws] CSSOMString escape(CSSOMString ident);
};

// https://drafts.csswg.org/css-conditional-3/#dom-css-supports
partial namespace CSS {
  boolean supports(CSSOMString property, CSSOMString value);
  boolean supports(CSSOMString conditionText);
};

// https://drafts.css-houdini.org/css-paint-api/#dom-css-paintworklet
partial namespace CSS {
    [SameObject, Pref="dom_worklet_enabled"] readonly attribute Worklet paintWorklet;
};

// https://drafts.css-houdini.org/css-properties-values-api/#the-registerproperty-function
dictionary PropertyDefinition {
  required DOMString name;
           DOMString syntax       = "*";
  required boolean   inherits;
           DOMString initialValue;
};

partial namespace CSS {
  [Throws] undefined registerProperty(PropertyDefinition definition);
};
