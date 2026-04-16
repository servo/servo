/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/sanitizer-api/#configobject
enum SanitizerPresets { "default" };
dictionary SetHTMLOptions {
  (Sanitizer or SanitizerConfig or SanitizerPresets) sanitizer = "default";
};
dictionary SetHTMLUnsafeOptions {
  (Sanitizer or SanitizerConfig or SanitizerPresets) sanitizer = {};
};

// https://wicg.github.io/sanitizer-api/#sanitizer
[Exposed=Window, Pref="dom_sanitizer_enabled"]
interface Sanitizer {
  [Throws] constructor(optional (SanitizerConfig or SanitizerPresets) configuration = "default");

  // Query configuration:
  SanitizerConfig get();

  // Modify a Sanitizer's lists and fields:
  // boolean allowElement(SanitizerElementWithAttributes element);
  // boolean removeElement(SanitizerElement element);
  // boolean replaceElementWithChildren(SanitizerElement element);
  // boolean allowAttribute(SanitizerAttribute attribute);
  // boolean removeAttribute(SanitizerAttribute attribute);
  // boolean setComments(boolean allow);
  // boolean setDataAttributes(boolean allow);

  // Remove markup that executes script.
  // boolean removeUnsafe();
};

// https://wicg.github.io/sanitizer-api/#config
dictionary SanitizerElementNamespace {
  required DOMString name;
  DOMString? _namespace = "http://www.w3.org/1999/xhtml";
};

// Used by "elements"
dictionary SanitizerElementNamespaceWithAttributes : SanitizerElementNamespace {
  sequence<SanitizerAttribute> attributes;
  sequence<SanitizerAttribute> removeAttributes;
};

typedef (DOMString or SanitizerElementNamespace) SanitizerElement;
typedef (DOMString or SanitizerElementNamespaceWithAttributes) SanitizerElementWithAttributes;

dictionary SanitizerAttributeNamespace {
  required DOMString name;
  DOMString? _namespace = null;
};
typedef (DOMString or SanitizerAttributeNamespace) SanitizerAttribute;

dictionary SanitizerConfig {
  sequence<SanitizerElementWithAttributes> elements;
  sequence<SanitizerElement> removeElements;
  sequence<SanitizerElement> replaceWithChildrenElements;

  sequence<SanitizerAttribute> attributes;
  sequence<SanitizerAttribute> removeAttributes;

  boolean comments;
  boolean dataAttributes;
};

