/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper module to build up a selector safely and efficiently.
//!
//! Our selector representation is designed to optimize matching, and has
//! several requirements:
//! * All simple selectors and combinators are stored inline in the same buffer
//!   as Component instances.
//! * We store the top-level compound selectors from right to left, i.e. in
//!   matching order.
//! * We store the simple selectors for each combinator from left to right, so
//!   that we match the cheaper simple selectors first.
//!
//! Meeting all these constraints without extra memmove traffic during parsing
//! is non-trivial. This module encapsulates those details and presents an
//! easy-to-use API for the parser.

use parser::{Combinator, Component, SelectorImpl};
use servo_arc::{Arc, HeaderWithLength, ThinArc};
use sink::Push;
use smallvec::{self, SmallVec};
use std::cmp;
use std::iter;
use std::ops::Add;
use std::ptr;
use std::slice;

/// Top-level SelectorBuilder struct. This should be stack-allocated by the
/// consumer and never moved (because it contains a lot of inline data that
/// would be slow to memmov).
///
/// After instantation, callers may call the push_simple_selector() and
/// push_combinator() methods to append selector data as it is encountered
/// (from left to right). Once the process is complete, callers should invoke
/// build(), which transforms the contents of the SelectorBuilder into a heap-
/// allocated Selector and leaves the builder in a drained state.
pub struct SelectorBuilder<Impl: SelectorImpl> {
    /// The entire sequence of simple selectors, from left to right, without combinators.
    ///
    /// We make this large because the result of parsing a selector is fed into a new
    /// Arc-ed allocation, so any spilled vec would be a wasted allocation. Also,
    /// Components are large enough that we don't have much cache locality benefit
    /// from reserving stack space for fewer of them.
    simple_selectors: SmallVec<[Component<Impl>; 32]>,
    /// The combinators, and the length of the compound selector to their left.
    combinators: SmallVec<[(Combinator, usize); 16]>,
    /// The length of the current compount selector.
    current_len: usize,
}

impl<Impl: SelectorImpl> Default for SelectorBuilder<Impl> {
    #[inline(always)]
    fn default() -> Self {
        SelectorBuilder {
            simple_selectors: SmallVec::new(),
            combinators: SmallVec::new(),
            current_len: 0,
        }
    }
}

impl<Impl: SelectorImpl> Push<Component<Impl>> for SelectorBuilder<Impl> {
    fn push(&mut self, value: Component<Impl>) {
        self.push_simple_selector(value);
    }
}

impl<Impl: SelectorImpl> SelectorBuilder<Impl> {
    /// Pushes a simple selector onto the current compound selector.
    #[inline(always)]
    pub fn push_simple_selector(&mut self, ss: Component<Impl>) {
        debug_assert!(!ss.is_combinator());
        self.simple_selectors.push(ss);
        self.current_len += 1;
    }

    /// Completes the current compound selector and starts a new one, delimited
    /// by the given combinator.
    #[inline(always)]
    pub fn push_combinator(&mut self, c: Combinator) {
        self.combinators.push((c, self.current_len));
        self.current_len = 0;
    }

    /// Returns true if no simple selectors have ever been pushed to this builder.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.simple_selectors.is_empty()
    }

    /// Returns true if combinators have ever been pushed to this builder.
    #[inline(always)]
    pub fn has_combinators(&self) -> bool {
        !self.combinators.is_empty()
    }

    /// Consumes the builder, producing a Selector.
    #[inline(always)]
    pub fn build(&mut self, parsed_pseudo: bool) -> ThinArc<SpecificityAndFlags, Component<Impl>> {
        // Compute the specificity and flags.
        let mut spec = SpecificityAndFlags(specificity(self.simple_selectors.iter()));
        if parsed_pseudo {
            spec.0 |= HAS_PSEUDO_BIT;
        }

        self.build_with_specificity_and_flags(spec)
    }


    /// Builds with an explicit SpecificityAndFlags. This is separated from build() so
    /// that unit tests can pass an explicit specificity.
    #[inline(always)]
    pub fn build_with_specificity_and_flags(&mut self, spec: SpecificityAndFlags)
                                            -> ThinArc<SpecificityAndFlags, Component<Impl>> {
        // First, compute the total number of Components we'll need to allocate
        // space for.
        let full_len = self.simple_selectors.len() + self.combinators.len();

        // Create the header.
        let header = HeaderWithLength::new(spec, full_len);

        // Create the Arc using an iterator that drains our buffers.

        // Use a raw pointer to be able to call set_len despite "borrowing" the slice.
        // This is similar to SmallVec::drain, but we use a slice here because
        // we’re gonna traverse it non-linearly.
        let raw_simple_selectors: *const [Component<Impl>] = &*self.simple_selectors;
        unsafe {
            // Panic-safety: if SelectorBuilderIter is not iterated to the end,
            // some simple selectors will safely leak.
            self.simple_selectors.set_len(0)
        }
        let (rest, current) = split_from_end(unsafe { &*raw_simple_selectors }, self.current_len);
        let iter = SelectorBuilderIter {
            current_simple_selectors: current.iter(),
            rest_of_simple_selectors: rest,
            combinators: self.combinators.drain().rev(),
        };

        Arc::into_thin(Arc::from_header_and_iter(header, iter))
    }
}

struct SelectorBuilderIter<'a, Impl: SelectorImpl> {
    current_simple_selectors: slice::Iter<'a, Component<Impl>>,
    rest_of_simple_selectors: &'a [Component<Impl>],
    combinators: iter::Rev<smallvec::Drain<'a, (Combinator, usize)>>,
}

impl<'a, Impl: SelectorImpl> ExactSizeIterator for SelectorBuilderIter<'a, Impl> {
    fn len(&self) -> usize {
        self.current_simple_selectors.len() +
        self.rest_of_simple_selectors.len() +
        self.combinators.len()
    }
}

impl<'a, Impl: SelectorImpl> Iterator for SelectorBuilderIter<'a, Impl> {
    type Item = Component<Impl>;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(simple_selector_ref) = self.current_simple_selectors.next() {
            // Move a simple selector out of this slice iterator.
            // This is safe because we’ve called SmallVec::set_len(0) above,
            // so SmallVec::drop won’t drop this simple selector.
            unsafe {
                Some(ptr::read(simple_selector_ref))
            }
        } else {
            self.combinators.next().map(|(combinator, len)| {
                let (rest, current) = split_from_end(self.rest_of_simple_selectors, len);
                self.rest_of_simple_selectors = rest;
                self.current_simple_selectors = current.iter();
                Component::Combinator(combinator)
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

fn split_from_end<T>(s: &[T], at: usize) -> (&[T], &[T]) {
    s.split_at(s.len() - at)
}

pub const HAS_PSEUDO_BIT: u32 = 1 << 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecificityAndFlags(pub u32);

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

fn specificity<Impl>(iter: slice::Iter<Component<Impl>>) -> u32
    where Impl: SelectorImpl
{
    complex_selector_specificity(iter).into()
}

fn complex_selector_specificity<Impl>(mut iter: slice::Iter<Component<Impl>>)
                                      -> Specificity
    where Impl: SelectorImpl
{
    fn simple_selector_specificity<Impl>(
        simple_selector: &Component<Impl>,
        specificity: &mut Specificity,
    )
    where
        Impl: SelectorImpl
    {
        match *simple_selector {
            Component::Combinator(..) => unreachable!(),
            // FIXME(emilio): Spec doesn't define any particular specificity for
            // ::slotted(), so apply the general rule for pseudos per:
            //
            // https://github.com/w3c/csswg-drafts/issues/1915
            //
            // Though other engines compute it dynamically, so maybe we should
            // do that instead, eventually.
            Component::Slotted(..) |
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
            Component::Empty | Component::Scope |
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
    for simple_selector in &mut iter {
        simple_selector_specificity(&simple_selector, &mut specificity);
    }
    specificity
}
