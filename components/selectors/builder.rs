/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parser::{Component, SelectorImpl, SelectorIter};
use std::cmp;
use std::ops::Add;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SpecificityAndFlags(pub u32);

pub const HAS_PSEUDO_BIT: u32 = 1 << 30;

impl SpecificityAndFlags {
    pub fn specificity(&self) -> u32 {
        self.0 & !HAS_PSEUDO_BIT
    }

    pub fn has_pseudo_element(&self) -> bool {
        (self.0 & HAS_PSEUDO_BIT) != 0
    }
}

const MAX_10BIT: u32 = (1u32 << 10) - 1;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Specificity {
    id_selectors: u32,
    class_like_selectors: u32,
    element_selectors: u32,
}

impl Add for Specificity {
    type Output = Specificity;

    fn add(self, rhs: Specificity) -> Specificity {
        Specificity {
            id_selectors: self.id_selectors + rhs.id_selectors,
            class_like_selectors:
                self.class_like_selectors + rhs.class_like_selectors,
            element_selectors:
                self.element_selectors + rhs.element_selectors,
        }
    }
}

impl Default for Specificity {
    fn default() -> Specificity {
        Specificity {
            id_selectors: 0,
            class_like_selectors: 0,
            element_selectors: 0,
        }
    }
}

impl From<u32> for Specificity {
    fn from(value: u32) -> Specificity {
        assert!(value <= MAX_10BIT << 20 | MAX_10BIT << 10 | MAX_10BIT);
        Specificity {
            id_selectors: value >> 20,
            class_like_selectors: (value >> 10) & MAX_10BIT,
            element_selectors: value & MAX_10BIT,
        }
    }
}

impl From<Specificity> for u32 {
    fn from(specificity: Specificity) -> u32 {
        cmp::min(specificity.id_selectors, MAX_10BIT) << 20
        | cmp::min(specificity.class_like_selectors, MAX_10BIT) << 10
        | cmp::min(specificity.element_selectors, MAX_10BIT)
    }
}

pub fn specificity<Impl>(iter: SelectorIter<Impl>) -> u32
    where Impl: SelectorImpl
{
    complex_selector_specificity(iter).into()
}

fn complex_selector_specificity<Impl>(mut iter: SelectorIter<Impl>)
                                      -> Specificity
    where Impl: SelectorImpl
{
    fn simple_selector_specificity<Impl>(simple_selector: &Component<Impl>,
                                         specificity: &mut Specificity)
        where Impl: SelectorImpl
    {
        match *simple_selector {
            Component::Combinator(..) => unreachable!(),
            Component::PseudoElement(..) |
            Component::LocalName(..) => {
                specificity.element_selectors += 1
            }
            Component::ID(..) => {
                specificity.id_selectors += 1
            }
            Component::Class(..) |
            Component::AttributeInNoNamespace { .. } |
            Component::AttributeInNoNamespaceExists { .. } |
            Component::AttributeOther(..) |

            Component::FirstChild | Component::LastChild |
            Component::OnlyChild | Component::Root |
            Component::Empty |
            Component::NthChild(..) |
            Component::NthLastChild(..) |
            Component::NthOfType(..) |
            Component::NthLastOfType(..) |
            Component::FirstOfType | Component::LastOfType |
            Component::OnlyOfType |
            Component::NonTSPseudoClass(..) => {
                specificity.class_like_selectors += 1
            }
            Component::ExplicitUniversalType |
            Component::ExplicitAnyNamespace |
            Component::ExplicitNoNamespace |
            Component::DefaultNamespace(..) |
            Component::Namespace(..) => {
                // Does not affect specificity
            }
            Component::Negation(ref negated) => {
                for ss in negated.iter() {
                    simple_selector_specificity(&ss, specificity);
                }
            }
        }
    }

    let mut specificity = Default::default();
    loop {
        for simple_selector in &mut iter {
            simple_selector_specificity(&simple_selector, &mut specificity);
        }
        if iter.next_sequence().is_none() {
            break;
        }
    }
    specificity
}
