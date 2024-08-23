/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#submitevent
[Exposed=Window]
interface SubmitEvent : Event {
    constructor(DOMString typeArg, optional SubmitEventInit eventInitDict = {});

    readonly attribute HTMLElement? submitter;
};

dictionary SubmitEventInit : EventInit {
    HTMLElement? submitter = null;
};
