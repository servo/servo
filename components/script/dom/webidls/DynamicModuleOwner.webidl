/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
 * This is defined for [`Dynamic Module`](https://html.spec.whatwg.org/multipage/#fetch-an-import()-module-script-graph)
 * so that we can hold a traceable owner for those dynamic modules which don't hold a owner.
 */

[LegacyNoInterfaceObject, Exposed=Window]
interface DynamicModuleOwner {
  readonly attribute Promise<any> promise;
};
