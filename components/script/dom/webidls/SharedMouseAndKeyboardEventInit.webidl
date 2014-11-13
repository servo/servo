/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#idl-def-SharedKeyboardAndMouseEventInit
dictionary SharedKeyboardAndMouseEventInit : UIEventInit {
    boolean ctrlKey = false;
    boolean shiftKey = false;
    boolean altKey = false;
    boolean metaKey = false;
    boolean keyModifierStateAltGraph = false;
    boolean keyModifierStateCapsLock = false;
    boolean keyModifierStateFn = false;
    boolean keyModifierStateFnLock = false;
    boolean keyModifierStateHyper = false;
    boolean keyModifierStateNumLock = false;
    boolean keyModifierStateOS = false;
    boolean keyModifierStateScrollLock = false;
    boolean keyModifierStateSuper = false;
    boolean keyModifierStateSymbol = false;
    boolean keyModifierStateSymbolLock = false;
};
