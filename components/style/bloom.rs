/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The style bloom filter is used as an optimization when matching deep
//! descendant selectors.

#![deny(missing_docs)]

use dom::{SendElement, TElement};
use selectors::bloom::BloomFilter;
use smallvec::SmallVec;

/// A struct that allows us to fast-reject deep descendant selectors avoiding
/// selector-matching.
///
/// This is implemented using a counting bloom filter, and it's a standard
/// optimization. See Gecko's `AncestorFilter`, and Blink's and WebKit's
/// `SelectorFilter`.
///
/// The constraints for Servo's style system are a bit different compared to
/// traditional style systems given Servo does a parallel breadth-first
/// traversal instead of a sequential depth-first traversal.
///
/// This implies that we need to track a bit more state than other browsers to
/// ensure we're doing the correct thing during the traversal, and being able to
/// apply this optimization effectively.
///
/// Concretely, we have a bloom filter instance per worker thread, and we track
/// the current DOM depth in order to find a common ancestor when it doesn't
/// match the previous element we've styled.
///
/// This is usually a pretty fast operation (we use to be one level deeper than
/// the previous one), but in the case of work-stealing, we may needed to push
/// and pop multiple elements.
///
/// See the `insert_parents_recovering`, where most of the magic happens.
///
/// Regarding thread-safety, this struct is safe because:
///
///  * We clear this after a restyle.
///  * The DOM shape and attributes (and every other thing we access here) are
///    immutable during a restyle.
///
pub struct StyleBloom<E: TElement> {
    /// The bloom filter per se.
    filter: Box<BloomFilter>,

    /// The stack of elements that this bloom filter contains.
    elements: Vec<SendElement<E>>,
}

fn each_relevant_element_hash<E, F>(element: E, mut f: F)
    where E: TElement,
          F: FnMut(u32),
{
    f(element.get_local_name().get_hash());
    f(element.get_namespace().get_hash());

    if let Some(id) = element.get_id() {
        f(id.get_hash());
    }

    // TODO: case-sensitivity depends on the document type and quirks mode.
    //
    // TODO(emilio): It's not clear whether that's relevant here though?
    // Classes and ids should be normalized already I think.
    element.each_class(|class| {
        f(class.get_hash())
    });
}

impl<E: TElement> StyleBloom<E> {
    /// Create an empty `StyleBloom`.
    pub fn new() -> Self {
        StyleBloom {
            filter: Box::new(BloomFilter::new()),
            elements: vec![],
        }
    }

    /// Return the bloom filter used properly by the `selectors` crate.
    pub fn filter(&self) -> &BloomFilter {
        &*self.filter
    }

    /// Push an element to the bloom filter, knowing that it's a child of the
    /// last element parent.
    pub fn push(&mut self, element: E) {
        if cfg!(debug_assertions) {
            if self.elements.is_empty() {
                assert!(element.traversal_parent().is_none());
            }
        }
        self.push_internal(element);
    }

    /// Same as `push`, but without asserting, in order to use it from
    /// `rebuild`.
    fn push_internal(&mut self, element: E) {
        each_relevant_element_hash(element, |hash| {
            self.filter.insert_hash(hash);
        });
        self.elements.push(unsafe { SendElement::new(element) });
    }

    /// Pop the last element in the bloom filter and return it.
    fn pop(&mut self) -> Option<E> {
        let popped = self.elements.pop().map(|el| *el);

        if let Some(popped) = popped {
            each_relevant_element_hash(popped, |hash| {
                self.filter.remove_hash(hash);
            })
        }

        popped
    }

    /// Returns true if the bloom filter is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the DOM depth of elements that can be correctly
    /// matched against the bloom filter (that is, the number of
    /// elements in our list).
    pub fn matching_depth(&self) -> usize {
        self.elements.len()
    }

    /// Clears the bloom filter.
    pub fn clear(&mut self) {
        self.filter.clear();
        self.elements.clear();
    }

    /// Rebuilds the bloom filter up to the parent of the given element.
    pub fn rebuild(&mut self, mut element: E) {
        self.clear();

        while let Some(parent) = element.traversal_parent() {
            self.push_internal(parent);
            element = parent;
        }

        // Put them in the order we expect, from root to `element`'s parent.
        self.elements.reverse();
    }

    /// In debug builds, asserts that all the parents of `element` are in the
    /// bloom filter.
    ///
    /// Goes away in release builds.
    pub fn assert_complete(&self, mut element: E) {
        if cfg!(debug_assertions) {
            let mut checked = 0;
            while let Some(parent) = element.traversal_parent() {
                assert_eq!(parent, *self.elements[self.elements.len() - 1 - checked]);
                element = parent;
                checked += 1;
            }
            assert_eq!(checked, self.elements.len());
        }
    }

    /// Get the element that represents the chain of things inserted
    /// into the filter right now.  That chain is the given element
    /// (if any) and its ancestors.
    #[inline]
    pub fn current_parent(&self) -> Option<E> {
        self.elements.last().map(|el| **el)
    }

    /// Insert the parents of an element in the bloom filter, trying to recover
    /// the filter if the last element inserted doesn't match.
    ///
    /// Gets the element depth in the dom, to make it efficient, or if not
    /// provided always rebuilds the filter from scratch.
    ///
    /// Returns the new bloom filter depth, that the traversal code is
    /// responsible to keep around if it wants to get an effective filter.
    pub fn insert_parents_recovering(&mut self,
                                     element: E,
                                     element_depth: usize)
    {
        // Easy case, we're in a different restyle, or we're empty.
        if self.elements.is_empty() {
            self.rebuild(element);
            return;
        }

        let traversal_parent = match element.traversal_parent() {
            Some(parent) => parent,
            None => {
                // Yay, another easy case.
                self.clear();
                return;
            }
        };

        if self.current_parent() == Some(traversal_parent) {
            // Ta da, cache hit, we're all done.
            return;
        }

        if element_depth == 0 {
            self.clear();
            return;
        }

        // We should've early exited above.
        debug_assert!(element_depth != 0,
                      "We should have already cleared the bloom filter");
        debug_assert!(!self.elements.is_empty(),
                      "How! We should've just rebuilt!");

        // Now the fun begins: We have the depth of the dom and the depth of the
        // last element inserted in the filter, let's try to find a common
        // parent.
        //
        // The current depth, that is, the depth of the last element inserted in
        // the bloom filter, is the number of elements _minus one_, that is: if
        // there's one element, it must be the root -> depth zero.
        let mut current_depth = self.elements.len() - 1;

        // If the filter represents an element too deep in the dom, we need to
        // pop ancestors.
        while current_depth > element_depth - 1 {
            self.pop().expect("Emilio is bad at math");
            current_depth -= 1;
        }

        // Now let's try to find a common parent in the bloom filter chain,
        // starting with traversal_parent.
        let mut common_parent = traversal_parent;
        let mut common_parent_depth = element_depth - 1;

        // Let's collect the parents we are going to need to insert once we've
        // found the common one.
        let mut parents_to_insert = SmallVec::<[E; 8]>::new();

        // If the bloom filter still doesn't have enough elements, the common
        // parent is up in the dom.
        while common_parent_depth > current_depth {
            // TODO(emilio): Seems like we could insert parents here, then
            // reverse the slice.
            parents_to_insert.push(common_parent);
            common_parent =
                common_parent.traversal_parent().expect("We were lied to");
            common_parent_depth -= 1;
        }

        // Now the two depths are the same.
        debug_assert_eq!(common_parent_depth, current_depth);

        // Happy case: The parents match, we only need to push the ancestors
        // we've collected and we'll never enter in this loop.
        //
        // Not-so-happy case: Parent's don't match, so we need to keep going up
        // until we find a common ancestor.
        //
        // Gecko currently models native anonymous content that conceptually
        // hangs off the document (such as scrollbars) as a separate subtree
        // from the document root.
        //
        // Thus it's possible with Gecko that we do not find any common
        // ancestor.
        while **self.elements.last().unwrap() != common_parent {
            parents_to_insert.push(common_parent);
            self.pop().unwrap();
            common_parent = match common_parent.traversal_parent() {
                Some(parent) => parent,
                None => {
                    debug_assert!(self.elements.is_empty());
                    if cfg!(feature = "gecko") {
                        break;
                    } else {
                        panic!("should have found a common ancestor");
                    }
                }
            }
        }

        // Now the parents match, so insert the stack of elements we have been
        // collecting so far.
        for parent in parents_to_insert.into_iter().rev() {
            self.push(parent);
        }

        debug_assert_eq!(self.elements.len(), element_depth);

        // We're done! Easy.
    }
}
