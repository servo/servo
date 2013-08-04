/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface please see
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface MouseEvent : UIEvent {
  readonly attribute long           screenX;
  readonly attribute long           screenY;
  readonly attribute long           clientX;
  readonly attribute long           clientY;
  readonly attribute boolean        ctrlKey;
  readonly attribute boolean        shiftKey;
  readonly attribute boolean        altKey;
  readonly attribute boolean        metaKey;
  readonly attribute unsigned short button;
  readonly attribute unsigned short buttons;
  readonly attribute EventTarget?   relatedTarget;
  // Deprecated in DOM Level 3:
  [Throws]
  void                              initMouseEvent(DOMString typeArg, 
                                                   boolean canBubbleArg, 
                                                   boolean cancelableArg, 
                                                   WindowProxy? viewArg, 
                                                   long detailArg, 
                                                   long screenXArg, 
                                                   long screenYArg, 
                                                   long clientXArg, 
                                                   long clientYArg, 
                                                   boolean ctrlKeyArg, 
                                                   boolean altKeyArg, 
                                                   boolean shiftKeyArg, 
                                                   boolean metaKeyArg, 
                                                   unsigned short buttonArg,
                                                   EventTarget? relatedTargetArg);
  // Introduced in DOM Level 3:
  boolean                           getModifierState(DOMString keyArg);
};


// Event Constructor Syntax:
[Constructor(DOMString typeArg, optional MouseEventInit mouseEventInitDict)]
partial interface MouseEvent
{
};

// Suggested initMouseEvent replacement initializer:
dictionary MouseEventInit {
  // Attributes from Event:
  boolean        bubbles       = false;
  boolean        cancelable    = false;

  // Attributes from UIEvent:
  WindowProxy?   view          = null;
  long           detail        = 0;

  // Attributes for MouseEvent:
  long           screenX       = 0;
  long           screenY       = 0;
  long           clientX       = 0;
  long           clientY       = 0;
  boolean        ctrlKey       = false;
  boolean        shiftKey      = false;
  boolean        altKey        = false;
  boolean        metaKey       = false;
  unsigned short button        = 0;
  // Note: "buttons" was not previously initializable through initMouseEvent!
  unsigned short buttons       = 0;
  EventTarget?   relatedTarget = null;
};
