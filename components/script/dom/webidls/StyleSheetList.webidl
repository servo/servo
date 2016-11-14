/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-stylesheetlist-interface
// [ArrayClass]
[Exposed=Window]
interface StyleSheetList {
  getter StyleSheet? item(unsigned long index);
  readonly attribute unsigned long length;
};
