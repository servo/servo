/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Property-based randomized testing for the core float layout algorithm.

#[macro_use]
extern crate lazy_static;

use euclid::num::Zero;
use layout::flow::float::{ClearSide, FloatBand, FloatBandNode, FloatBandTree, FloatContext};
use layout::flow::float::{FloatSide, PlacementInfo};
use layout::geom::flow_relative::{Rect, Vec2};
use quickcheck::{Arbitrary, Gen};
use std::f32;
use std::ops::Range;
use std::panic::{self, PanicInfo};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::u32;
use style::values::computed::Length;

lazy_static! {
    static ref PANIC_HOOK_MUTEX: Mutex<()> = Mutex::new(());
}

// Suppresses panic messages. Some tests need to fail and we don't want them to spam the console.
// Note that, because the panic hook is process-wide, tests that are expected to fail might
// suppress panic messages from other failing tests. To work around this, run failing tests one at
// a time or use only a single test thread.
struct PanicMsgSuppressor<'a> {
    #[allow(dead_code)]
    mutex_guard: MutexGuard<'a, ()>,
    prev_hook: Option<Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send>>,
}

impl<'a> PanicMsgSuppressor<'a> {
    fn new(mutex_guard: MutexGuard<'a, ()>) -> PanicMsgSuppressor<'a> {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| ()));
        PanicMsgSuppressor {
            mutex_guard,
            prev_hook: Some(prev_hook),
        }
    }
}

impl<'a> Drop for PanicMsgSuppressor<'a> {
    fn drop(&mut self) {
        panic::set_hook(self.prev_hook.take().unwrap())
    }
}

// AA tree helpers

#[derive(Clone, Debug)]
struct FloatBandWrapper(FloatBand);

impl Arbitrary for FloatBandWrapper {
    fn arbitrary<G>(generator: &mut G) -> FloatBandWrapper
    where
        G: Gen,
    {
        let top: u32 = Arbitrary::arbitrary(generator);
        let left: Option<u32> = Arbitrary::arbitrary(generator);
        let right: Option<u32> = Arbitrary::arbitrary(generator);
        FloatBandWrapper(FloatBand {
            top: Length::new(top as f32),
            left: left.map(|value| Length::new(value as f32)),
            right: right.map(|value| Length::new(value as f32)),
        })
    }
}

#[derive(Clone, Debug)]
struct FloatRangeInput {
    start_index: u32,
    band_count: u32,
    side: FloatSide,
    length: u32,
}

impl Arbitrary for FloatRangeInput {
    fn arbitrary<G>(generator: &mut G) -> FloatRangeInput
    where
        G: Gen,
    {
        let start_index: u32 = Arbitrary::arbitrary(generator);
        let band_count: u32 = Arbitrary::arbitrary(generator);
        let is_left: bool = Arbitrary::arbitrary(generator);
        let length: u32 = Arbitrary::arbitrary(generator);
        FloatRangeInput {
            start_index,
            band_count,
            side: if is_left {
                FloatSide::Left
            } else {
                FloatSide::Right
            },
            length,
        }
    }
}

// AA tree predicates

fn check_node_ordering(node: &FloatBandNode) {
    let mid = node.band.top;
    if let Some(ref left) = node.left.0 {
        assert!(left.band.top < mid);
    }
    if let Some(ref right) = node.right.0 {
        assert!(right.band.top > mid);
    }
    if let Some(ref left) = node.left.0 {
        check_node_ordering(left);
    }
    if let Some(ref right) = node.right.0 {
        check_node_ordering(right);
    }
}

// https://en.wikipedia.org/wiki/AA_tree#Balancing_rotations
fn check_node_balance(node: &FloatBandNode) {
    // 1. The level of every leaf node is one.
    if node.left.0.is_none() && node.right.0.is_none() {
        assert_eq!(node.level, 1);
    }
    // 2. The level of every left child is exactly one less than that of its parent.
    if let Some(ref left) = node.left.0 {
        assert_eq!(left.level, node.level - 1);
    }
    // 3. The level of every right child is equal to or one less than that of its parent.
    if let Some(ref right) = node.right.0 {
        assert!(right.level == node.level || right.level == node.level - 1);
    }
    // 4. The level of every right grandchild is strictly less than that of its grandparent.
    if let Some(ref right) = node.right.0 {
        if let Some(ref right_right) = right.right.0 {
            assert!(right_right.level < node.level);
        }
    }
    // 5. Every node of level greater than one has two children.
    if node.level > 1 {
        assert!(node.left.0.is_some() && node.right.0.is_some());
    }
}

fn check_tree_ordering(tree: FloatBandTree) {
    if let Some(ref root) = tree.root.0 {
        check_node_ordering(root);
    }
}

fn check_tree_balance(tree: FloatBandTree) {
    if let Some(ref root) = tree.root.0 {
        check_node_balance(root);
    }
}

fn check_tree_find(tree: &FloatBandTree, block_position: Length, sorted_bands: &[FloatBand]) {
    let found_band = tree
        .find(block_position)
        .expect("Couldn't find the band in the tree!");
    let reference_band_index = sorted_bands
        .iter()
        .position(|band| band.top > block_position)
        .expect("Couldn't find the reference band!") -
        1;
    let reference_band = &sorted_bands[reference_band_index];
    assert_eq!(found_band.top, reference_band.top);
    assert_eq!(found_band.left, reference_band.left);
    assert_eq!(found_band.right, reference_band.right);
}

fn check_tree_find_next(tree: &FloatBandTree, block_position: Length, sorted_bands: &[FloatBand]) {
    let found_band = tree
        .find_next(block_position)
        .expect("Couldn't find the band in the tree!");
    let reference_band_index = sorted_bands
        .iter()
        .position(|band| band.top > block_position)
        .expect("Couldn't find the reference band!");
    let reference_band = &sorted_bands[reference_band_index];
    assert_eq!(found_band.top, reference_band.top);
    assert_eq!(found_band.left, reference_band.left);
    assert_eq!(found_band.right, reference_band.right);
}

fn check_node_range_setting(
    node: &FloatBandNode,
    block_range: &Range<Length>,
    side: FloatSide,
    value: Length,
) {
    if node.band.top >= block_range.start && node.band.top < block_range.end {
        match side {
            FloatSide::Left => assert!(node.band.left.unwrap() >= value),
            FloatSide::Right => assert!(node.band.right.unwrap() <= value),
        }
    }

    if let Some(ref left) = node.left.0 {
        check_node_range_setting(left, block_range, side, value)
    }
    if let Some(ref right) = node.right.0 {
        check_node_range_setting(right, block_range, side, value)
    }
}

fn check_tree_range_setting(
    tree: &FloatBandTree,
    block_range: &Range<Length>,
    side: FloatSide,
    value: Length,
) {
    if let Some(ref root) = tree.root.0 {
        check_node_range_setting(root, block_range, side, value)
    }
}

// AA tree unit tests

// Tests that the tree is a properly-ordered binary tree.
#[test]
fn test_tree_ordering() {
    let f: fn(Vec<FloatBandWrapper>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>) {
        let mut tree = FloatBandTree::new();
        for FloatBandWrapper(band) in bands {
            tree = tree.insert(band);
        }
        check_tree_ordering(tree);
    }
}

// Tests that the tree is balanced (i.e. AA tree invariants are maintained).
#[test]
fn test_tree_balance() {
    let f: fn(Vec<FloatBandWrapper>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>) {
        let mut tree = FloatBandTree::new();
        for FloatBandWrapper(band) in bands {
            tree = tree.insert(band);
        }
        check_tree_balance(tree);
    }
}

// Tests that the `find()` method works.
#[test]
fn test_tree_find() {
    let f: fn(Vec<FloatBandWrapper>, Vec<u32>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>, lookups: Vec<u32>) {
        let mut bands: Vec<FloatBand> = bands.into_iter().map(|band| band.0).collect();
        bands.push(FloatBand {
            top: Length::zero(),
            left: None,
            right: None,
        });
        bands.push(FloatBand {
            top: Length::new(f32::INFINITY),
            left: None,
            right: None,
        });
        let mut tree = FloatBandTree::new();
        for ref band in &bands {
            tree = tree.insert((*band).clone());
        }
        bands.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());
        for lookup in lookups {
            check_tree_find(&tree, Length::new(lookup as f32), &bands);
        }
    }
}

// Tests that the `find_next()` method works.
#[test]
fn test_tree_find_next() {
    let f: fn(Vec<FloatBandWrapper>, Vec<u32>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>, lookups: Vec<u32>) {
        let mut bands: Vec<FloatBand> = bands.into_iter().map(|band| band.0).collect();
        bands.push(FloatBand {
            top: Length::zero(),
            left: None,
            right: None,
        });
        bands.push(FloatBand {
            top: Length::new(f32::INFINITY),
            left: None,
            right: None,
        });
        bands.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());
        bands.dedup_by(|a, b| a.top == b.top);
        let mut tree = FloatBandTree::new();
        for ref band in &bands {
            tree = tree.insert((*band).clone());
        }
        for lookup in lookups {
            check_tree_find_next(&tree, Length::new(lookup as f32), &bands);
        }
    }
}

// Tests that `set_range()` works.
#[test]
fn test_tree_range_setting() {
    let f: fn(Vec<FloatBandWrapper>, Vec<FloatRangeInput>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>, ranges: Vec<FloatRangeInput>) {
        let mut tree = FloatBandTree::new();
        for FloatBandWrapper(ref band) in &bands {
            tree = tree.insert((*band).clone());
        }

        let mut tops: Vec<Length> = bands.iter().map(|band| band.0.top).collect();
        tops.push(Length::new(f32::INFINITY));
        tops.sort_by(|a, b| a.px().partial_cmp(&b.px()).unwrap());

        for range in ranges {
            let start = range.start_index.min(tops.len() as u32 - 1);
            let end = (range.start_index + range.length).min(tops.len() as u32 - 1);
            let block_range = tops[start as usize]..tops[end as usize];
            let length = Length::new(range.length as f32);
            let new_tree = tree.set_range(&block_range, range.side, length);
            check_tree_range_setting(&new_tree, &block_range, range.side, length);
        }
    }
}

// Float predicates

#[derive(Clone, Debug)]
struct FloatInput {
    // Information needed to place the float.
    info: PlacementInfo,
    // The float may be placed no higher than this line. This simulates the effect of line boxes
    // per CSS 2.1 § 9.5.1 rule 6.
    ceiling: u32,
    /// The distance from the logical left side of the block formatting context to the logical
    /// left side of the current containing block.
    left_wall: Length,
    /// The distance from the logical *left* side of the block formatting context to the logical
    /// right side of this object's containing block.
    right_wall: Length,
}

impl Arbitrary for FloatInput {
    fn arbitrary<G>(generator: &mut G) -> FloatInput
    where
        G: Gen,
    {
        let width: u32 = Arbitrary::arbitrary(generator);
        let height: u32 = Arbitrary::arbitrary(generator);
        let is_left: bool = Arbitrary::arbitrary(generator);
        let ceiling: u32 = Arbitrary::arbitrary(generator);
        let left_wall: u32 = Arbitrary::arbitrary(generator);
        let right_wall: u32 = Arbitrary::arbitrary(generator);
        let clear: u8 = Arbitrary::arbitrary(generator);
        FloatInput {
            info: PlacementInfo {
                size: Vec2 {
                    inline: Length::new(width as f32),
                    block: Length::new(height as f32),
                },
                side: if is_left {
                    FloatSide::Left
                } else {
                    FloatSide::Right
                },
                clear: new_clear_side(clear),
            },
            ceiling,
            left_wall: Length::new(left_wall as f32),
            right_wall: Length::new(right_wall as f32),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = FloatInput>> {
        let mut this = (*self).clone();
        let mut shrunk = false;
        if let Some(inline_size) = self.info.size.inline.px().shrink().next() {
            this.info.size.inline = Length::new(inline_size);
            shrunk = true;
        }
        if let Some(block_size) = self.info.size.block.px().shrink().next() {
            this.info.size.block = Length::new(block_size);
            shrunk = true;
        }
        if let Some(clear_side) = (self.info.clear as u8).shrink().next() {
            this.info.clear = new_clear_side(clear_side);
            shrunk = true;
        }
        if let Some(left_wall) = self.left_wall.px().shrink().next() {
            this.left_wall = Length::new(left_wall);
            shrunk = true;
        }
        if let Some(right_wall) = self.right_wall.px().shrink().next() {
            this.right_wall = Length::new(right_wall);
            shrunk = true;
        }
        if let Some(ceiling) = self.ceiling.shrink().next() {
            this.ceiling = ceiling;
            shrunk = true;
        }
        if shrunk {
            quickcheck::single_shrinker(this)
        } else {
            quickcheck::empty_shrinker()
        }
    }
}

fn new_clear_side(value: u8) -> ClearSide {
    match value & 3 {
        0 => ClearSide::None,
        1 => ClearSide::Left,
        2 => ClearSide::Right,
        _ => ClearSide::Both,
    }
}

#[derive(Clone)]
struct FloatPlacement {
    float_context: FloatContext,
    placed_floats: Vec<PlacedFloat>,
}

// Information about the placement of a float.
#[derive(Clone)]
struct PlacedFloat {
    origin: Vec2<Length>,
    info: PlacementInfo,
    ceiling: Length,
    left_wall: Length,
    right_wall: Length,
}

impl Drop for FloatPlacement {
    fn drop(&mut self) {
        if !thread::panicking() {
            return;
        }

        // Dump the float context for debugging.
        eprintln!("Failing float placement:");
        for placed_float in &self.placed_floats {
            eprintln!(
                "   * {:?} @ {:?}, T {:?} L {:?} R {:?}",
                placed_float.info,
                placed_float.origin,
                placed_float.ceiling,
                placed_float.left_wall,
                placed_float.right_wall,
            );
        }
        eprintln!("Bands:\n{:?}\n", self.float_context.bands);
    }
}

impl PlacedFloat {
    fn rect(&self) -> Rect<Length> {
        Rect {
            start_corner: self.origin.clone(),
            size: self.info.size.clone(),
        }
    }
}

impl FloatPlacement {
    fn place(floats: Vec<FloatInput>) -> FloatPlacement {
        let mut float_context = FloatContext::new();
        let mut placed_floats = vec![];
        for float in floats {
            let ceiling = Length::new(float.ceiling as f32);
            float_context.lower_ceiling(ceiling);
            float_context.left_wall = float.left_wall;
            float_context.right_wall = float.right_wall;
            let placement_offset = Vec2 {
                inline: float.left_wall,
                block: Length::zero(),
            };
            placed_floats.push(PlacedFloat {
                origin: &float_context.add_float(&float.info) + &placement_offset,
                info: float.info,
                ceiling,
                left_wall: float.left_wall,
                right_wall: float.right_wall,
            })
        }
        FloatPlacement {
            float_context,
            placed_floats,
        }
    }
}

// From CSS 2.1 § 9.5.1 [1].
//
// [1]: https://www.w3.org/TR/CSS2/visuren.html#float-position

// 1. The left outer edge of a left-floating box may not be to the left of the left edge of its
//    containing block. An analogous rule holds for right-floating elements.
fn check_floats_rule_1(placement: &FloatPlacement) {
    for placed_float in &placement.placed_floats {
        match placed_float.info.side {
            FloatSide::Left => assert!(placed_float.origin.inline >= placed_float.left_wall),
            FloatSide::Right => {
                assert!(placed_float.rect().max_inline_position() <= placed_float.right_wall)
            },
        }
    }
}

// 2. If the current box is left-floating, and there are any left-floating boxes generated by
//    elements earlier in the source document, then for each such earlier box, either the left
//    outer edge of the current box must be to the right of the right outer edge of the earlier
//    box, or its top must be lower than the bottom of the earlier box. Analogous rules hold for
//    right-floating boxes.
fn check_floats_rule_2(placement: &FloatPlacement) {
    for (this_float_index, this_float) in placement.placed_floats.iter().enumerate() {
        for prev_float in &placement.placed_floats[0..this_float_index] {
            match (this_float.info.side, prev_float.info.side) {
                (FloatSide::Left, FloatSide::Left) => {
                    assert!(
                        this_float.origin.inline >= prev_float.rect().max_inline_position() ||
                            this_float.origin.block >= prev_float.rect().max_block_position()
                    );
                },
                (FloatSide::Right, FloatSide::Right) => {
                    assert!(
                        this_float.rect().max_inline_position() <= prev_float.origin.inline ||
                            this_float.origin.block >= prev_float.rect().max_block_position()
                    );
                },
                (FloatSide::Left, FloatSide::Right) | (FloatSide::Right, FloatSide::Left) => {},
            }
        }
    }
}

// 3. The right outer edge of a left-floating box may not be to the right of the left outer edge of
//    any right-floating box that is next to it. Analogous rules hold for right-floating elements.
fn check_floats_rule_3(placement: &FloatPlacement) {
    for (this_float_index, this_float) in placement.placed_floats.iter().enumerate() {
        for other_float in &placement.placed_floats[0..this_float_index] {
            // This logic to check intersection is complicated by the fact that we need to treat
            // zero-height floats later in the document as "next to" floats earlier in the
            // document. Otherwise we might end up with a situation like:
            //
            //    <div id="a" style="float: left; width: 32px; height: 32px"></div>
            //    <div id="b" style="float: right; width: 0px; height: 0px"></div>
            //
            // Where the top of `b` should probably be 32px per Rule 3, but unless this distinction
            // is made the top of `b` could legally be 0px.
            if this_float.origin.block >= other_float.rect().max_block_position() ||
                (this_float.info.size.block == Length::zero() &&
                    this_float.rect().max_block_position() < other_float.origin.block) ||
                (this_float.info.size.block > Length::zero() &&
                    this_float.rect().max_block_position() <= other_float.origin.block)
            {
                continue;
            }

            match (this_float.info.side, other_float.info.side) {
                (FloatSide::Left, FloatSide::Right) => {
                    assert!(this_float.rect().max_inline_position() <= other_float.origin.inline);
                },
                (FloatSide::Right, FloatSide::Left) => {
                    assert!(this_float.origin.inline >= other_float.rect().max_inline_position());
                },
                (FloatSide::Left, FloatSide::Left) | (FloatSide::Right, FloatSide::Right) => {},
            }
        }
    }
}

// 4. A floating box's outer top may not be higher than the top of its containing block. When the
//    float occurs between two collapsing margins, the float is positioned as if it had an
//    otherwise empty anonymous block parent taking part in the flow. The position of such a parent
//    is defined by the rules in the section on margin collapsing.
fn check_floats_rule_4(placement: &FloatPlacement) {
    for placed_float in &placement.placed_floats {
        assert!(placed_float.origin.block >= Length::zero());
    }
}

// 5. The outer top of a floating box may not be higher than the outer top of any block or floated
//    box generated by an element earlier in the source document.
fn check_floats_rule_5(placement: &FloatPlacement) {
    let mut block_position = Length::zero();
    for placed_float in &placement.placed_floats {
        assert!(placed_float.origin.block >= block_position);
        block_position = placed_float.origin.block;
    }
}

// 6. The outer top of an element's floating box may not be higher than the top of any line-box
//    containing a box generated by an element earlier in the source document.
fn check_floats_rule_6(placement: &FloatPlacement) {
    for placed_float in &placement.placed_floats {
        assert!(placed_float.origin.block >= placed_float.ceiling);
    }
}

// 7. A left-floating box that has another left-floating box to its left may not have its right
//    outer edge to the right of its containing block's right edge. (Loosely: a left float may not
//    stick out at the right edge, unless it is already as far to the left as possible.) An
//    analogous rule holds for right-floating elements.
fn check_floats_rule_7(placement: &FloatPlacement) {
    for (placed_float_index, placed_float) in placement.placed_floats.iter().enumerate() {
        // Only consider floats that stick out.
        match placed_float.info.side {
            FloatSide::Left => {
                if placed_float.rect().max_inline_position() <= placed_float.right_wall {
                    continue;
                }
            },
            FloatSide::Right => {
                if placed_float.origin.inline >= placed_float.left_wall {
                    continue;
                }
            },
        }

        // Make sure there are no previous floats to the left or right.
        for prev_float in &placement.placed_floats[0..placed_float_index] {
            assert!(
                prev_float.info.side != placed_float.info.side ||
                    prev_float.rect().max_block_position() <= placed_float.origin.block ||
                    prev_float.origin.block >= placed_float.rect().max_block_position()
            );
        }
    }
}

// 8. A floating box must be placed as high as possible.
fn check_floats_rule_8(floats_and_perturbations: Vec<(FloatInput, u32)>) {
    let floats = floats_and_perturbations
        .iter()
        .map(|&(ref float, _)| (*float).clone())
        .collect();
    let placement = FloatPlacement::place(floats);

    for (float_index, &(_, perturbation)) in floats_and_perturbations.iter().enumerate() {
        if perturbation == 0 {
            continue;
        }

        let mut placement = placement.clone();
        placement.placed_floats[float_index].origin.block =
            placement.placed_floats[float_index].origin.block - Length::new(perturbation as f32);

        let result = {
            let mutex_guard = PANIC_HOOK_MUTEX.lock().unwrap();
            let _suppressor = PanicMsgSuppressor::new(mutex_guard);
            panic::catch_unwind(|| check_basic_float_rules(&placement))
        };
        assert!(result.is_err());
    }
}

// 9. A left-floating box must be put as far to the left as possible, a right-floating box as far
//    to the right as possible. A higher position is preferred over one that is further to the
//    left/right.
fn check_floats_rule_9(floats_and_perturbations: Vec<(FloatInput, u32)>) {
    let floats = floats_and_perturbations
        .iter()
        .map(|&(ref float, _)| (*float).clone())
        .collect();
    let placement = FloatPlacement::place(floats);

    for (float_index, &(_, perturbation)) in floats_and_perturbations.iter().enumerate() {
        if perturbation == 0 {
            continue;
        }

        let mut placement = placement.clone();
        {
            let mut placed_float = &mut placement.placed_floats[float_index];
            let perturbation = Length::new(perturbation as f32);
            match placed_float.info.side {
                FloatSide::Left => {
                    placed_float.origin.inline = placed_float.origin.inline - perturbation
                },
                FloatSide::Right => {
                    placed_float.origin.inline = placed_float.origin.inline + perturbation
                },
            }
        }

        let result = {
            let mutex_guard = PANIC_HOOK_MUTEX.lock().unwrap();
            let _suppressor = PanicMsgSuppressor::new(mutex_guard);
            panic::catch_unwind(|| check_basic_float_rules(&placement))
        };
        assert!(result.is_err());
    }
}

// From CSS 2.1 § 9.5.2 (https://www.w3.org/TR/CSS2/visuren.html#propdef-clear):
//
// 10. The top outer edge of the float must be below the bottom outer edge of all earlier
//     left-floating boxes (in the case of 'clear: left'), or all earlier right-floating boxes (in
//     the case of 'clear: right'), or both ('clear: both').
fn check_floats_rule_10(placement: &FloatPlacement) {
    let mut block_position = Length::zero();
    for placed_float in &placement.placed_floats {
        assert!(placed_float.origin.block >= block_position);
        block_position = placed_float.origin.block;
    }

    for (this_float_index, this_float) in placement.placed_floats.iter().enumerate() {
        if this_float.info.clear == ClearSide::None {
            continue;
        }

        for other_float in &placement.placed_floats[0..this_float_index] {
            // This logic to check intersection is complicated by the fact that we need to treat
            // zero-height floats later in the document as "next to" floats earlier in the
            // document. Otherwise we might end up with a situation like:
            //
            //    <div id="a" style="float: left; width: 32px; height: 32px"></div>
            //    <div id="b" style="float: right; width: 0px; height: 0px"></div>
            //
            // Where the top of `b` should probably be 32px per Rule 3, but unless this distinction
            // is made the top of `b` could legally be 0px.
            if this_float.origin.block >= other_float.rect().max_block_position() ||
                (this_float.info.size.block == Length::zero() &&
                    this_float.rect().max_block_position() < other_float.origin.block) ||
                (this_float.info.size.block > Length::zero() &&
                    this_float.rect().max_block_position() <= other_float.origin.block)
            {
                continue;
            }

            match this_float.info.clear {
                ClearSide::Left => assert_ne!(other_float.info.side, FloatSide::Left),
                ClearSide::Right => assert_ne!(other_float.info.side, FloatSide::Right),
                ClearSide::Both => assert!(false),
                ClearSide::None => unreachable!(),
            }
        }
    }
}

// Checks that rule 1-7 and rule 10 hold (i.e. all rules that don't specify that floats are placed
// "as far as possible" in some direction).
fn check_basic_float_rules(placement: &FloatPlacement) {
    check_floats_rule_1(placement);
    check_floats_rule_2(placement);
    check_floats_rule_3(placement);
    check_floats_rule_4(placement);
    check_floats_rule_5(placement);
    check_floats_rule_6(placement);
    check_floats_rule_7(placement);
    check_floats_rule_10(placement);
}

// Float unit tests

#[test]
fn test_floats_rule_1() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_1(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_2() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_2(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_3() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_3(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_4() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_4(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_5() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_5(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_6() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_6(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_7() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_7(&FloatPlacement::place(floats));
    }
}

#[test]
fn test_floats_rule_8() {
    let f: fn(Vec<(FloatInput, u32)>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<(FloatInput, u32)>) {
        check_floats_rule_8(floats);
    }
}

#[test]
fn test_floats_rule_9() {
    let f: fn(Vec<(FloatInput, u32)>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<(FloatInput, u32)>) {
        check_floats_rule_9(floats);
    }
}

#[test]
fn test_floats_rule_10() {
    let f: fn(Vec<FloatInput>) = check;
    quickcheck::quickcheck(f);
    fn check(floats: Vec<FloatInput>) {
        check_floats_rule_10(&FloatPlacement::place(floats));
    }
}
