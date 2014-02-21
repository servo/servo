/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* FIXME WindowProxy doesn't actually have an interface according to the spec,
         but I'm not sure how to do fallible unwrapping without this, since
         we lack Gecko's XPCOM querying facilities. */
interface WindowProxy {
};