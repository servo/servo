/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/worklets/#workletglobalscope
// TODO: The spec IDL doesn't make this a subclass of EventTarget
//       https://github.com/whatwg/html/issues/2611
[Exposed=Worklet]
interface WorkletGlobalScope: GlobalScope {
};
