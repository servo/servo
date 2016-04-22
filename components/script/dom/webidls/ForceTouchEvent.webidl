/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://developer.apple.com/library/mac/documentation/AppleApplications/Conceptual/SafariJSProgTopics/RespondingtoForceTouchEventsfromJavaScript.html

/**
 * Events: (copy/paste from apple.com)
 *
 *  webkitmouseforcewillbegin: This event occurs immediately before the mousedown event. It allows you to
 *   prevent the default system behavior, such as displaying a dictionary window when force clicking on a
 *   word, in order to perform a custom action instead. To prevent the default system behavior, call the
 *   preventDefault() method on the event.
 *  webkitmouseforcedown: This event occurs after the mousedown event, once enough force has been applied
 *   to register as a force click. The user receives haptic feedback representing the force click when this
 *   event occurs.
 *  webkitmouseforceup: This event occurs after a webkitmouseforcedown event, once enough force has been
 *   released to exit the force click operation. The user receives haptic feedback representing the exit
 *   from force click when this event occurs.
 *  webkitmouseforcechanged: This event occurs whenever a change in trackpad force is detected between the
 *   mousedown and mouseup events.
 *
 */


[Pref="dom.forcetouch.enabled"]
interface ForceTouchEvent : UIEvent {
    // Represents the amount of force required to perform a regular click.
    readonly attribute float SERVO_FORCE_AT_MOUSE_DOWN;
    // Represents the force required to perform a force click.
    readonly attribute float SERVO_FORCE_AT_FORCE_MOUSE_DOWN;
    // force level
    readonly attribute float servoForce;
};
