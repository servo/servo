/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/uievents/#dictdef-eventmodifierinit
dictionary EventModifierInit : UIEventInit {
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
