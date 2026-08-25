/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Private interfaces that are only used for internal Servo usage
// like about: pages.

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=Window,
Func="ServoInternals::is_servo_internal"]
interface ServoInternals {
    Promise<object> reportMemory();
    undefined garbageCollectAllContexts();

    sequence<USVString> preferenceList();
    [Throws] USVString preferenceType(USVString name);
    [Throws] any defaultPreferenceValue(USVString name);
    [Throws] any getPreference(USVString name);
    [Throws] USVString getStringPreference(USVString name);
    [Throws] long long getIntPreference(USVString name);
    [Throws] boolean getBoolPreference(USVString name);
    undefined setStringPreference(USVString name, USVString value);
    undefined setIntPreference(USVString name, long long value);
    undefined setBoolPreference(USVString name, boolean value);
};

partial interface Navigator {
    [Func="ServoInternals::is_servo_internal"]
    readonly attribute ServoInternals servo;
};
