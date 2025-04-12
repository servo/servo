/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */


// https://streams.spec.whatwg.org/#ts-default-controller-class-definition

[Exposed=*]
interface TransformStreamDefaultController {
  readonly attribute unrestricted double? desiredSize;
  [Throws] undefined enqueue(optional any chunk);
  [Throws] undefined error(optional any reason);
  [Throws] undefined terminate();
};



// The TransformUnderlyingSource interface is entirely internal to Servo, and should not be accessible to
// web pages.
[LegacyNoInterfaceObject, Exposed=(Window,Worker)]
// Need to escape "TransformUnderlyingSource" so it's treated as an identifier.
interface _TransformUnderlyingSource {
};
