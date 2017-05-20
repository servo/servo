/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/worklets/#worklet
[Exposed=(Window)]
interface Worklet {
    [NewObject] Promise<void> addModule(USVString moduleURL, optional WorkletOptions options);
};

dictionary WorkletOptions {
    RequestCredentials credentials = "omit";
};
