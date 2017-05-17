/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#mutationobserver
 */

// https://dom.spec.whatwg.org/#mutationobserver
[Pref="dom.mutation_observer.enabled", Constructor(MutationCallback callback)]
interface MutationObserver {
    [Throws]
    void observe(Node target, optional MutationObserverInit options);
    //void disconnect();
    //sequence<MutationRecord> takeRecords();
};

callback MutationCallback = void (sequence<MutationRecord> mutations, MutationObserver observer);

dictionary MutationObserverInit {
    boolean childList = false;
    boolean attributes;
    boolean characterData;
    boolean subtree = false;
    boolean attributeOldValue;
    boolean characterDataOldValue;
    sequence<DOMString> attributeFilter;
};
