/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The style bloom filter is used as an optimization when matching deep
//! descendant selectors.

use dom::{TNode, TElement, UnsafeNode};
use matching::MatchMethods;
use selectors::bloom::BloomFilter;

pub struct StyleBloom {
    /// The bloom filter per se.
    filter: Box<BloomFilter>,

    /// The stack of elements that this bloom filter contains. These unsafe
    /// nodes are guaranteed to be elements.
    ///
    /// Note that the use we do for them is safe, since the data we access from
    /// them is completely read-only during restyling.
    elements: Vec<UnsafeNode>,

    /// A monotonic counter incremented which each reflow in order to invalidate
    /// the bloom filter if appropriate.
    generation: u32,
}

impl StyleBloom {
    pub fn new(generation: u32) -> Self {
        StyleBloom {
            filter: Box::new(BloomFilter::new()),
            elements: vec![],
            generation: generation,
        }
    }

    pub fn filter(&self) -> &BloomFilter {
        &*self.filter
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn maybe_pop<E>(&mut self, element: E)
        where E: TElement + MatchMethods
    {
        if self.elements.last() == Some(&element.as_node().to_unsafe()) {
            self.pop::<E>().unwrap();
        }
    }

    /// Push an element to the bloom filter, knowing that it's a child of the
    /// last element parent.
    pub fn push<E>(&mut self, element: E)
        where E: TElement + MatchMethods,
    {
        if cfg!(debug_assertions) {
            if self.elements.is_empty() {
                assert!(element.parent_element().is_none());
            }
        }
        element.insert_into_bloom_filter(&mut *self.filter);
        self.elements.push(element.as_node().to_unsafe());
    }

    /// Pop the last element in the bloom filter and return it.
    fn pop<E>(&mut self) -> Option<E>
        where E: TElement + MatchMethods,
    {
        let popped =
            self.elements.pop().map(|unsafe_node| {
                let parent = unsafe {
                    E::ConcreteNode::from_unsafe(&unsafe_node)
                };
                parent.as_element().unwrap()
            });
        if let Some(popped) = popped {
            popped.remove_from_bloom_filter(&mut self.filter);
        }

        popped
    }

    fn clear(&mut self) {
        self.filter.clear();
        self.elements.clear();
    }

    fn rebuild<E>(&mut self, mut element: E) -> usize
        where E: TElement + MatchMethods,
    {
        self.clear();

        while let Some(parent) = element.parent_element() {
            parent.insert_into_bloom_filter(&mut *self.filter);
            self.elements.push(parent.as_node().to_unsafe());
            element = parent;
        }

        // Put them in the order we expect, from root to `element`'s parent.
        self.elements.reverse();
        return self.elements.len();
    }

    /// In debug builds, asserts that all the parents of `element` are in the
    /// bloom filter.
    pub fn assert_complete<E>(&self, mut element: E)
        where E: TElement,
    {
        if cfg!(debug_assertions) {
            let mut checked = 0;
            while let Some(parent) = element.parent_element() {
                assert_eq!(parent.as_node().to_unsafe(),
                           self.elements[self.elements.len() - 1 - checked]);
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
    pub fn insert_parents_recovering<E>(&mut self,
                                        element: E,
                                        element_depth: Option<usize>,
                                        generation: u32)
                                        -> usize
        where E: TElement,
    {
        // Easy case, we're in a different restyle, or we're empty.
        if self.generation != generation || self.elements.is_empty() {
            self.generation = generation;
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

        let unsafe_parent = parent_element.as_node().to_unsafe();
        if self.elements.last() == Some(&unsafe_parent) {
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
        while current_depth >= element_depth - 1 {
            self.pop::<E>().expect("Emilio is bad at math");
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
        while *self.elements.last().unwrap() != common_parent.as_node().to_unsafe() {
            parents_to_insert.push(common_parent);
            common_parent =
                common_parent.parent_element().expect("We were lied again?");
            self.pop::<E>().unwrap();
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
