/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/aria/#dom-ariamixin
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface mixin ARIAMixin {
  [CEReactions] attribute DOMString? role;
  [CEReactions] attribute DOMString? ariaAtomic;
  [CEReactions] attribute DOMString? ariaAutoComplete;
  [CEReactions] attribute DOMString? ariaBrailleLabel;
  [CEReactions] attribute DOMString? ariaBrailleRoleDescription;
  [CEReactions] attribute DOMString? ariaBusy;
  [CEReactions] attribute DOMString? ariaChecked;
  [CEReactions] attribute DOMString? ariaColCount;
  [CEReactions] attribute DOMString? ariaColIndex;
  [CEReactions] attribute DOMString? ariaColIndexText;
  [CEReactions] attribute DOMString? ariaColSpan;
  [CEReactions] attribute DOMString? ariaCurrent;
  [CEReactions] attribute DOMString? ariaDescription;
  [CEReactions] attribute DOMString? ariaDisabled;
  [CEReactions] attribute DOMString? ariaExpanded;
  [CEReactions] attribute DOMString? ariaHasPopup;
  [CEReactions] attribute DOMString? ariaHidden;
  [CEReactions] attribute DOMString? ariaInvalid;
  [CEReactions] attribute DOMString? ariaKeyShortcuts;
  [CEReactions] attribute DOMString? ariaLabel;
  [CEReactions] attribute DOMString? ariaLevel;
  [CEReactions] attribute DOMString? ariaLive;
  [CEReactions] attribute DOMString? ariaModal;
  [CEReactions] attribute DOMString? ariaMultiLine;
  [CEReactions] attribute DOMString? ariaMultiSelectable;
  [CEReactions] attribute DOMString? ariaOrientation;
  [CEReactions] attribute DOMString? ariaPlaceholder;
  [CEReactions] attribute DOMString? ariaPosInSet;
  [CEReactions] attribute DOMString? ariaPressed;
  [CEReactions] attribute DOMString? ariaReadOnly;
  [CEReactions] attribute DOMString? ariaRelevant;
  [CEReactions] attribute DOMString? ariaRequired;
  [CEReactions] attribute DOMString? ariaRoleDescription;
  [CEReactions] attribute DOMString? ariaRowCount;
  [CEReactions] attribute DOMString? ariaRowIndex;
  [CEReactions] attribute DOMString? ariaRowIndexText;
  [CEReactions] attribute DOMString? ariaRowSpan;
  [CEReactions] attribute DOMString? ariaSelected;
  [CEReactions] attribute DOMString? ariaSetSize;
  [CEReactions] attribute DOMString? ariaSort;
  [CEReactions] attribute DOMString? ariaValueMax;
  [CEReactions] attribute DOMString? ariaValueMin;
  [CEReactions] attribute DOMString? ariaValueNow;
  [CEReactions] attribute DOMString? ariaValueText;
};
