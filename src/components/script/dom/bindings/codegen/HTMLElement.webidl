/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/ and
 * http://dev.w3.org/csswg/cssom-view/
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLElement : Element {
  // metadata attributes
           attribute DOMString title;
           attribute DOMString lang;
  //         attribute boolean translate;
  [SetterThrows, Pure]
           attribute DOMString dir;
  /*[Constant]
    readonly attribute DOMStringMap dataset;*/

  // microdata 
  /*[SetterThrows, Pure]
           attribute boolean itemScope;
  [PutForwards=value,Constant] readonly attribute DOMSettableTokenList itemType;
  [SetterThrows, Pure]
           attribute DOMString itemId;
  [PutForwards=value,Constant] readonly attribute DOMSettableTokenList itemRef;
  [PutForwards=value,Constant] readonly attribute DOMSettableTokenList itemProp;*/
  /*[Constant]
    readonly attribute HTMLPropertiesCollection properties;*/
  [Throws]
           attribute any itemValue;

  // user interaction
  [SetterThrows, Pure]
           attribute boolean hidden;
  void click();
  [SetterThrows, Pure]
           attribute long tabIndex;
  [Throws]
  void focus();
  [Throws]
  void blur();
  [SetterThrows, Pure]
           attribute DOMString accessKey;
  [Pure]
  readonly attribute DOMString accessKeyLabel;
  [SetterThrows, Pure]
           attribute boolean draggable;
  //[PutForwards=value] readonly attribute DOMSettableTokenList dropzone;
  [SetterThrows, Pure]
           attribute DOMString contentEditable;
  [Pure]
  readonly attribute boolean isContentEditable;
  /*[Pure]
    readonly attribute HTMLMenuElement? contextMenu;*/
  //[SetterThrows]
  //         attribute HTMLMenuElement? contextMenu;
  [SetterThrows, Pure]
           attribute boolean spellcheck;

  // command API
  //readonly attribute DOMString? commandType;
  //readonly attribute DOMString? commandLabel;
  //readonly attribute DOMString? commandIcon;
  //readonly attribute boolean? commandHidden;
  //readonly attribute boolean? commandDisabled;
  //readonly attribute boolean? commandChecked;

  // styling
  /*[PutForwards=cssText, Constant]
    readonly attribute CSSStyleDeclaration style;*/

  // Mozilla specific stuff
  // FIXME Bug 810677 Move className from HTMLElement to Element
           attribute DOMString className;

  /*[SetterThrows]
           attribute EventHandler oncopy;
  [SetterThrows]
           attribute EventHandler oncut;
  [SetterThrows]
  attribute EventHandler onpaste;*/
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-htmlelement-interface
partial interface HTMLElement {
  // CSSOM things are not [Pure] because they can flush
  readonly attribute Element? offsetParent;
  readonly attribute long offsetTop;
  readonly attribute long offsetLeft;
  readonly attribute long offsetWidth;
  readonly attribute long offsetHeight;
};

/*[NoInterfaceObject]
interface TouchEventHandlers {
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchstart;
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchend;
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchmove;
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchenter;
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchleave;
  [SetterThrows,Func="nsGenericHTMLElement::TouchEventsEnabled"]
           attribute EventHandler ontouchcancel;
};*/

/*HTMLElement implements GlobalEventHandlers;
HTMLElement implements NodeEventHandlers;
HTMLElement implements TouchEventHandlers;*/
