/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#mutationrecord
 */

// https://dom.spec.whatwg.org/#mutationrecord
[Pref="dom.mutation_observer.enabled", Exposed=Window]
interface MutationRecord {
    readonly attribute DOMString type;
    [SameObject]
    readonly attribute Node target;
    //[SameObject]
    //readonly attribute NodeList addedNodes;
    //[SameObject]
    //readonly attribute NodeList removedNodes;
    //readonly attribute Node? previousSibling;
    //readonly attribute Node? nextSibling;
    //readonly attribute DOMString? attributeName;
    //readonly attribute DOMString? attributeNamespace;
    //readonly attribute DOMString? oldValue;
};
