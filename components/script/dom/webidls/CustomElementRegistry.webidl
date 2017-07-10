/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#customelementregistry
[Pref="dom.customelements.enabled"]
interface CustomElementRegistry {
  [Throws, CEReactions]
  void define(DOMString name, Function constructor_, optional ElementDefinitionOptions options);

  any get(DOMString name);

  Promise<void> whenDefined(DOMString name);
};

dictionary ElementDefinitionOptions {
  DOMString extends;
};
