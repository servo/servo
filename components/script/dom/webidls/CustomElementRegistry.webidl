/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#customelementregistry
[Exposed=Window, Pref="dom.custom_elements.enabled"]
interface CustomElementRegistry {
  [Throws, CEReactions]
  void define(DOMString name, CustomElementConstructor constructor_, optional ElementDefinitionOptions options = {});

  any get(DOMString name);

  Promise<CustomElementConstructor> whenDefined(DOMString name);

  [CEReactions] void upgrade(Node root);
};

callback CustomElementConstructor = HTMLElement();

dictionary ElementDefinitionOptions {
  DOMString extends;
};
