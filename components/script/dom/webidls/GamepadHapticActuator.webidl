/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.w3.org/TR/gamepad/#gamepadhapticactuator-interface
[Exposed=Window, Pref="dom.gamepad.enabled"]
interface GamepadHapticActuator {
  /* [SameObject] */ readonly attribute /* FrozenArray<GamepadHapticEffectType> */ any effects;
  Promise<GamepadHapticsResult> playEffect(
    GamepadHapticEffectType type,
    optional GamepadEffectParameters params = {}
  );
  Promise<GamepadHapticsResult> reset();
};

// https://www.w3.org/TR/gamepad/#gamepadhapticsresult-enum
enum GamepadHapticsResult {
  "complete",
  "preempted"
};

// https://www.w3.org/TR/gamepad/#dom-gamepadhapticeffecttype
enum GamepadHapticEffectType {
  "dual-rumble"
};

// https://www.w3.org/TR/gamepad/#dom-gamepadeffectparameters
dictionary GamepadEffectParameters {
  unsigned long long duration = 0;
  unsigned long long startDelay = 0;
  double strongMagnitude = 0.0;
  double weakMagnitude = 0.0;
};
