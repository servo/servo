/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The style bloom filter is used as an optimization when matching deep
//! descendant selectors.

use dom::{SendElement, TElement};
use matching::MatchMethods;
use selectors::bloom::BloomFilter;

pub struct StyleBloom<E: TElement> {
    /// The bloom filter per se.
    filter: Box<BloomFilter>,

    /// The stack of elements that this bloom filter contains.
    elements: Vec<SendElement<E>>,
}

impl<E: TElement> StyleBloom<E> {
    pub fn new() -> Self {
        StyleBloom {
            filter: Box::new(BloomFilter::new()),
            elements: vec![],
        }
    }

    pub fn filter(&self) -> &BloomFilter {
        &*self.filter
    }

    pub fn maybe_pop(&mut self, element: E) {
        if self.elements.last().map(|el| **el) == Some(element) {
            self.pop().unwrap();
        }
    }

    /// Push an element to the bloom filter, knowing that it's a child of the
    /// last element parent.
    pub fn push(&mut self, element: E) {
        if cfg!(debug_assertions) {
            if self.elements.is_empty() {
                assert!(element.parent_element().is_none());
            }
        }
        element.insert_into_bloom_filter(&mut *self.filter);
        self.elements.push(unsafe { SendElement::new(element) });
    }

    /// Pop the last element in the bloom filter and return it.
    fn pop(&mut self) -> Option<E> {
        let popped = self.elements.pop().map(|el| *el);
        if let Some(popped) = popped {
            popped.remove_from_bloom_filter(&mut self.filter);
        }

        popped
    }

    fn clear(&mut self) {
        self.filter.clear();
        self.elements.clear();
    }

    fn rebuild(&mut self, mut element: E) -> usize {
        self.clear();

        while let Some(parent) = element.parent_element() {
            parent.insert_into_bloom_filter(&mut *self.filter);
            self.elements.push(unsafe { SendElement::new(parent) });
            element = parent;
        }

        // Put them in the order we expect, from root to `element`'s parent.
        self.elements.reverse();
        return self.elements.len();
    }

    /// In debug builds, asserts that all the parents of `element` are in the
    /// bloom filter.
    pub fn assert_complete(&self, mut element: E) {
        if cfg!(debug_assertions) {
            let mut checked = 0;
            while let Some(parent) = element.parent_element() {
                assert_eq!(parent, *self.elements[self.elements.len() - 1 - checked]);
                element = parent;
                checked += 1;
            }
            assert_eq!(checked, self.elements.len());
        }
    }

    /// Insert the parents of an element in the bloom filter, trying to recover
    /// the filter if the last element inserted doesn't match.
    ///
    /// Gets the element depth in the dom, to make it efficient, or if not
    /// provided always rebuilds the filter from scratch.
    ///
    /// Returns the new bloom filter depth.
    pub fn insert_parents_recovering(&mut self,
                                     element: E,
                                     element_depth: Option<usize>)
                                     -> usize
    {
        // Easy case, we're in a different restyle, or we're empty.
        if self.elements.is_empty() {
            return self.rebuild(element);
        }

        let parent_element = match element.parent_element() {
            Some(parent) => parent,
            None => {
                // Yay, another easy case.
                self.clear();
                return 0;
            }
        };

        if self.elements.last().map(|el| **el) == Some(parent_element) {
            // Ta da, cache hit, we're all done.
            return self.elements.len();
        }

        let element_depth = match element_depth {
            Some(depth) => depth,
            // If we don't know the depth of `element`, we'd rather don't try
            // fixing up the bloom filter, since it's quadratic.
            None => {
                return self.rebuild(element);
            }
        };

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
        // starting with parent_element.
        let mut common_parent = parent_element;
        let mut common_parent_depth = element_depth - 1;

        // Let's collect the parents we are going to need to insert once we've
        // found the common one.
        let mut parents_to_insert = vec![];

        // If the bloom filter still doesn't have enough elements, the common
        // parent is up in the dom.
        while common_parent_depth > current_depth {
            // TODO(emilio): Seems like we could insert parents here, then
            // reverse the slice.
            parents_to_insert.push(common_parent);
            common_parent =
                common_parent.parent_element().expect("We were lied");
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
        // Gecko currently models native anonymous content that conceptually hangs
        // off the document (such as scrollbars) as a separate subtree from the
        // document root.  Thus it's possible with Gecko that we do not find any
        // common ancestor.
        while **self.elements.last().unwrap() != common_parent {
            parents_to_insert.push(common_parent);
            self.pop().unwrap();
            common_parent = match common_parent.parent_element() {
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
        return self.elements.len();
    }
}
