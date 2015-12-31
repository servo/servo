/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level interface to CSS selector matching.

#![allow(unsafe_code)]

use animation;
use msg::ParseErrorReporter;
use script::layout_interface::Animation;
use selectors::bloom::BloomFilter;
use selectors::parser::PseudoElement;
use selectors::{Element};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use style::context::SharedStyleContext;
use style::data::PrivateStyleData;
use style::dom::{TElement, TNode, TRestyleDamage};
use style::matching::{ApplicableDeclarations, ApplicableDeclarationsCache};
use style::matching::{StyleSharingCandidate, StyleSharingCandidateCache};
use style::properties::{ComputedValues, cascade};
use style::selector_matching::{DeclarationBlock, Stylist};
use util::arc_ptr_eq;
use util::opts;

/// The results of attempting to share a style.
pub enum StyleSharingResult<ConcreteRestyleDamage: TRestyleDamage> {
    /// We didn't find anybody to share the style with.
    CannotShare,
    /// The node's style can be shared. The integer specifies the index in the LRU cache that was
    /// hit and the damage that was done.
    StyleWasShared(usize, ConcreteRestyleDamage),
}

trait PrivateMatchMethods<'ln>: TNode<'ln> {
    fn cascade_node_pseudo_element(&self,
                                   context: &SharedStyleContext,
                                   parent_style: Option<&Arc<ComputedValues>>,
                                   applicable_declarations: &[DeclarationBlock],
                                   style: &mut Option<Arc<ComputedValues>>,
                                   applicable_declarations_cache:
                                    &mut ApplicableDeclarationsCache,
                                   new_animations_sender: &Mutex<Sender<Animation>>,
                                   shareable: bool,
                                   animate_properties: bool)
                                   -> Self::ConcreteRestyleDamage {
        let mut cacheable = true;
        if animate_properties {
            cacheable = !self.update_animations_for_cascade(context, style) && cacheable;
        }

        let mut this_style;
        match parent_style {
            Some(ref parent_style) => {
                let cache_entry = applicable_declarations_cache.find(applicable_declarations);
                let cached_computed_values = match cache_entry {
                    None => None,
                    Some(ref style) => Some(&**style),
                };
                let (the_style, is_cacheable) = cascade(context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        Some(&***parent_style),
                                                        cached_computed_values,
                                                        context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
            None => {
                let (the_style, is_cacheable) = cascade(context.viewport_size,
                                                        applicable_declarations,
                                                        shareable,
                                                        None,
                                                        None,
                                                        context.error_reporter.clone());
                cacheable = cacheable && is_cacheable;
                this_style = the_style
            }
        };

        // Trigger transitions if necessary. This will reset `this_style` back to its old value if
        // it did trigger a transition.
        if animate_properties {
            if let Some(ref style) = *style {
                let animations_started =
                    animation::start_transitions_if_applicable(new_animations_sender,
                                                               self.opaque(),
                                                               &**style,
                                                               &mut this_style);
                cacheable = cacheable && !animations_started
            }
        }

        // Calculate style difference.
        let this_style = Arc::new(this_style);
        let damage = Self::ConcreteRestyleDamage::compute(style, &*this_style);

        // Cache the resolved style if it was cacheable.
        if cacheable {
            applicable_declarations_cache.insert(applicable_declarations.to_vec(),
                                                 this_style.clone());
        }

        // Write in the final style and return the damage done to our caller.
        *style = Some(this_style);
        damage
    }

    fn update_animations_for_cascade(&self,
                                     context: &SharedStyleContext,
                                     style: &mut Option<Arc<ComputedValues>>)
                                     -> bool {
        let style = match *style {
            None => return false,
            Some(ref mut style) => style,
        };

        // Finish any expired transitions.
        let this_opaque = self.opaque();
        let had_animations_to_expire;
        {
            let all_expired_animations = context.expired_animations.read().unwrap();
            let animations_to_expire = all_expired_animations.get(&this_opaque);
            had_animations_to_expire = animations_to_expire.is_some();
            if let Some(ref animations) = animations_to_expire {
                for animation in *animations {
                    animation.property_animation.update(&mut *Arc::make_mut(style), 1.0);
                }
            }
        }

        if had_animations_to_expire {
            context.expired_animations.write().unwrap().remove(&this_opaque);
        }

        // Merge any running transitions into the current style, and cancel them.
        let had_running_animations = context.running_animations
                                            .read()
                                            .unwrap()
                                            .get(&this_opaque)
                                            .is_some();
        if had_running_animations {
            let mut all_running_animations = context.running_animations.write().unwrap();
            for running_animation in all_running_animations.get(&this_opaque).unwrap() {
                animation::update_style_for_animation::<Self::ConcreteRestyleDamage>(running_animation, style, None);
            }
            all_running_animations.remove(&this_opaque);
        }

        had_animations_to_expire || had_running_animations
    }
}

impl<'ln, N: TNode<'ln>> PrivateMatchMethods<'ln> for N {}

trait PrivateElementMatchMethods<'le>: TElement<'le> {
    fn share_style_with_candidate_if_possible(&self,
                                              parent_node: Option<Self::ConcreteNode>,
                                              candidate: &StyleSharingCandidate)
                                              -> Option<Arc<ComputedValues>> {
        let parent_node = match parent_node {
            Some(ref parent_node) if parent_node.as_element().is_some() => parent_node,
            Some(_) | None => return None,
        };

        let parent_data: Option<&PrivateStyleData> = unsafe {
            parent_node.borrow_data_unchecked().map(|d| &*d)
        };
        match parent_data {
            Some(parent_data_ref) => {
                // Check parent style.
                let parent_style = (*parent_data_ref).style.as_ref().unwrap();
                if !arc_ptr_eq(parent_style, &candidate.parent_style) {
                    return None
                }

                // Check tag names, classes, etc.
                if !candidate.can_share_style_with(self) {
                    return None
                }

                return Some(candidate.style.clone())
            }
            _ => {}
        }

        None
    }
}

impl<'le, E: TElement<'le>> PrivateElementMatchMethods<'le> for E {}

pub trait ElementMatchMethods<'le> : TElement<'le> {
    fn match_element(&self,
                     stylist: &Stylist,
                     parent_bf: Option<&BloomFilter>,
                     applicable_declarations: &mut ApplicableDeclarations)
                     -> bool {
        let style_attribute = self.style_attribute().as_ref();

        applicable_declarations.normal_shareable =
            stylist.push_applicable_declarations(self,
                                                 parent_bf,
                                                 style_attribute,
                                                 None,
                                                 &mut applicable_declarations.normal);
        stylist.push_applicable_declarations(self,
                                             parent_bf,
                                             None,
                                             Some(PseudoElement::Before),
                                             &mut applicable_declarations.before);
        stylist.push_applicable_declarations(self,
                                             parent_bf,
                                             None,
                                             Some(PseudoElement::After),
                                             &mut applicable_declarations.after);

        applicable_declarations.normal_shareable &&
        applicable_declarations.before.is_empty() &&
        applicable_declarations.after.is_empty()
    }

    /// Attempts to share a style with another node. This method is unsafe because it depends on
    /// the `style_sharing_candidate_cache` having only live nodes in it, and we have no way to
    /// guarantee that at the type system level yet.
    unsafe fn share_style_if_possible(&self,
                                      style_sharing_candidate_cache:
                                        &mut StyleSharingCandidateCache,
                                      parent: Option<Self::ConcreteNode>)
                                      -> StyleSharingResult<<Self::ConcreteNode as TNode<'le>>::ConcreteRestyleDamage> {
        if opts::get().disable_share_style_cache {
            return StyleSharingResult::CannotShare
        }

        if self.style_attribute().is_some() {
            return StyleSharingResult::CannotShare
        }
        if self.get_attr(&ns!(), &atom!("id")).is_some() {
            return StyleSharingResult::CannotShare
        }

        for (i, &(ref candidate, ())) in style_sharing_candidate_cache.iter().enumerate() {
            match self.share_style_with_candidate_if_possible(parent.clone(), candidate) {
                Some(shared_style) => {
                    // Yay, cache hit. Share the style.
                    let node = self.as_node();
                    let style = &mut node.mutate_data().unwrap().style;
                    let damage = <<Self as TElement<'le>>::ConcreteNode as TNode<'le>>::ConcreteRestyleDamage::compute(style, &*shared_style);
                    *style = Some(shared_style);
                    return StyleSharingResult::StyleWasShared(i, damage)
                }
                None => {}
            }
        }

        StyleSharingResult::CannotShare
    }
}

impl<'le, E: TElement<'le>> ElementMatchMethods<'le> for E {}

pub trait MatchMethods<'ln> : TNode<'ln> {
    // The below two functions are copy+paste because I can't figure out how to
    // write a function which takes a generic function. I don't think it can
    // be done.
    //
    // Ideally, I'd want something like:
    //
    //   > fn with_really_simple_selectors(&self, f: <H: Hash>|&H|);


    // In terms of `SimpleSelector`s, these two functions will insert and remove:
    //   - `SimpleSelector::LocalName`
    //   - `SimpleSelector::Namepace`
    //   - `SimpleSelector::ID`
    //   - `SimpleSelector::Class`

    /// Inserts and removes the matching `Descendant` selectors from a bloom
    /// filter. This is used to speed up CSS selector matching to remove
    /// unnecessary tree climbs for `Descendant` queries.
    ///
    /// A bloom filter of the local names, namespaces, IDs, and classes is kept.
    /// Therefore, each node must have its matching selectors inserted _after_
    /// its own selector matching and _before_ its children start.
    fn insert_into_bloom_filter(&self, bf: &mut BloomFilter) {
        // Only elements are interesting.
        if let Some(element) = self.as_element() {
            bf.insert(element.get_local_name());
            bf.insert(element.get_namespace());
            element.get_id().map(|id| bf.insert(&id));

            // TODO: case-sensitivity depends on the document type and quirks mode
            element.each_class(|class| bf.insert(class));
        }
    }

    /// After all the children are done css selector matching, this must be
    /// called to reset the bloom filter after an `insert`.
    fn remove_from_bloom_filter(&self, bf: &mut BloomFilter) {
        // Only elements are interesting.
        if let Some(element) = self.as_element() {
            bf.remove(element.get_local_name());
            bf.remove(element.get_namespace());
            element.get_id().map(|id| bf.remove(&id));

            // TODO: case-sensitivity depends on the document type and quirks mode
            element.each_class(|class| bf.remove(class));
        }
    }

    unsafe fn cascade_node(&self,
                           context: &SharedStyleContext,
                           parent: Option<Self>,
                           applicable_declarations: &ApplicableDeclarations,
                           applicable_declarations_cache: &mut ApplicableDeclarationsCache,
                           new_animations_sender: &Mutex<Sender<Animation>>) {
        // Get our parent's style. This must be unsafe so that we don't touch the parent's
        // borrow flags.
        //
        // FIXME(pcwalton): Isolate this unsafety into the `wrapper` module to allow
        // enforced safe, race-free access to the parent style.
        let parent_style = match parent {
            None => None,
            Some(parent_node) => {
                let parent_style = (*parent_node.borrow_data_unchecked().unwrap()).style.as_ref().unwrap();
                Some(parent_style)
            }
        };

        if self.is_text_node() {
            // Text nodes get a copy of the parent style. This ensures
            // that during fragment construction any non-inherited
            // CSS properties (such as vertical-align) are correctly
            // set on the fragment(s).
            let mut data_ref = self.mutate_data().unwrap();
            let mut data = &mut *data_ref;
            let cloned_parent_style = parent_style.unwrap().clone();
            data.style = Some(cloned_parent_style);
        } else {
            let mut damage;
            {
                let mut data_ref = self.mutate_data().unwrap();
                let mut data = &mut *data_ref;
                damage = self.cascade_node_pseudo_element(
                    context,
                    parent_style,
                    &applicable_declarations.normal,
                    &mut data.style,
                    applicable_declarations_cache,
                    new_animations_sender,
                    applicable_declarations.normal_shareable,
                    true);
                if !applicable_declarations.before.is_empty() {
                    damage = damage | self.cascade_node_pseudo_element(
                        context,
                        Some(data.style.as_ref().unwrap()),
                        &*applicable_declarations.before,
                        &mut data.before_style,
                        applicable_declarations_cache,
                        new_animations_sender,
                        false,
                        false);
                }
                if !applicable_declarations.after.is_empty() {
                    damage = damage | self.cascade_node_pseudo_element(
                        context,
                        Some(data.style.as_ref().unwrap()),
                        &*applicable_declarations.after,
                        &mut data.after_style,
                        applicable_declarations_cache,
                        new_animations_sender,
                        false,
                        false);
                }
            }

            // This method needs to borrow the data as mutable, so make sure data_ref goes out of
            // scope first.
            self.set_restyle_damage(damage);
        }
    }
}

impl<'ln, N: TNode<'ln>> MatchMethods<'ln> for N {}
