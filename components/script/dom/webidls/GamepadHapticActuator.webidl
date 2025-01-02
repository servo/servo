/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/gamepad/#gamepadhapticactuator-interface
[Exposed=Window, Pref="dom.gamepad.enabled"]
interface GamepadHapticActuator {
  /* [SameObject] */ readonly attribute /* FrozenArray<GamepadHapticEffectType> */ any effects;
  [NewObject]
  Promise<GamepadHapticsResult> playEffect(
    GamepadHapticEffectType type,
    optional GamepadEffectParameters params = {}
  );
  [NewObject]
  Promise<GamepadHapticsResult> reset();
};

// https://w3c.github.io/gamepad/#gamepadhapticsresult-enum
enum GamepadHapticsResult {
  "complete",
  "preempted"
};

// https://w3c.github.io/gamepad/#dom-gamepadhapticeffecttype
enum GamepadHapticEffectType {
  "dual-rumble",
  "trigger-rumble"
};

// https://w3c.github.io/gamepad/#dom-gamepadeffectparameters
dictionary GamepadEffectParameters {
  unsigned long long duration = 0;
  unsigned long long startDelay = 0;
  double strongMagnitude = 0.0;
  double weakMagnitude = 0.0;
  double leftTrigger = 0.0;
  double rightTrigger = 0.0;
};
