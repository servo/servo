/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#mutationobserver
 */

// https://dom.spec.whatwg.org/#mutationobserver
[Exposed=Window, Pref="dom.mutation_observer.enabled"]
interface MutationObserver {
    [Throws] constructor(MutationCallback callback);
    [Throws]
    undefined observe(Node target, optional MutationObserverInit options = {});
    undefined disconnect();
    sequence<MutationRecord> takeRecords();
};

callback MutationCallback = undefined (sequence<MutationRecord> mutations, MutationObserver observer);

dictionary MutationObserverInit {
    boolean childList = false;
    boolean attributes;
    boolean characterData;
    boolean subtree = false;
    boolean attributeOldValue;
    boolean characterDataOldValue;
    sequence<DOMString> attributeFilter;
};
