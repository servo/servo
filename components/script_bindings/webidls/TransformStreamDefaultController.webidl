/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://streams.spec.whatwg.org/#ts-default-controller-class-definition
 */

[Exposed=*]
interface TransformStreamDefaultController {
  readonly attribute unrestricted double? desiredSize;
  [Throws] undefined enqueue(optional any chunk);
  [Throws] undefined error(optional any reason);
  [Throws] undefined terminate();
};
