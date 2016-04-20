
interface CSSStyleSheet : StyleSheet {
    //readonly attribute CSSRule? ownerRule;
    //[SameObject] readonly attribute CSSRuleList cssRules;
    unsigned long insertRule(DOMString rule, unsigned long index);
    void deleteRule(unsigned long index);
};