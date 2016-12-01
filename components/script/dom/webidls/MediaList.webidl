/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-medialist-interface
// [LegacyArrayClass]
interface MediaList {
  [TreatNullAs=EmptyString] /* stringifier */ attribute DOMString mediaText;
  readonly attribute unsigned long length;
  getter DOMString? item(unsigned long index);
  void appendMedium(DOMString medium);
  void deleteMedium(DOMString medium);
};
