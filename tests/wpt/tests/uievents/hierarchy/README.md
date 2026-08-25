Verify the Class Hierarchy
==========================

Make sure the events inherit from the correct interfaces:
    e.g., UIEvent > MouseEvent

Requires manual and automated tests
* manually create event and verify hierarchy
* WebDriver create the event and verify hierarchy

UIEvent
 * load, unload, abort, error, select, resize, scroll
 * Note: some event types may be dropped given that they don't appear to be UIEvents by other specs that define them.

FocusEvent
 * blur, focus, focusin, focusout
 * blur and focus are handled in HTML5
 * but they aren't sure if focusin/out are needed: see bug: https://www.w3.org/Bugs/Public/show_bug.cgi?id=25877

MouseEvent
 * click, dblclick, mousedown, mouseenter, mouseleave, mousemove, mouseout, mouseover, mouseup

WheelEvent
 * wheel

KeyboardEvent
 * keydown, keyup
 * need to show interaction with beforeinput and input, which are in the Editing spec

CompositionEvent
 * compositionstart
 * compositionupdate
 * compositionend
 * need to show interaction with the keyboard events: keydown, keyup
