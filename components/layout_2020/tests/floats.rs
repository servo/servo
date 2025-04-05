/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Property-based randomized testing for the core float layout algorithm.

use std::f32::INFINITY;
use std::ops::Range;
use std::panic::{self, PanicHookInfo};
use std::sync::{Mutex, MutexGuard};
use std::{thread, u32};

use app_units::Au;
use euclid::num::Zero;
use layout_2020::flow::float::{
    Clear, ContainingBlockPositionInfo, FloatBand, FloatBandNode, FloatBandTree, FloatContext,
    FloatSide, PlacementInfo,
};
use layout_2020::geom::{LogicalRect, LogicalVec2};
use quickcheck::{Arbitrary, Gen};

static PANIC_HOOK_MUTEX: Mutex<()> = Mutex::new(());

// Suppresses panic messages. Some tests need to fail and we don't want them to spam the console.
// Note that, because the panic hook is process-wide, tests that are expected to fail might
// suppress panic messages from other failing tests. To work around this, run failing tests one at
// a time or use only a single test thread.
struct PanicMsgSuppressor<'a> {
    #[allow(dead_code)]
    mutex_guard: MutexGuard<'a, ()>,
    prev_hook: Option<Box<dyn Fn(&PanicHookInfo<'_>) + 'static + Sync + Send>>,
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
    fn arbitrary(generator: &mut Gen) -> FloatBandWrapper {
        let top: u32 = u32::arbitrary(generator);
        let inline_start: Option<u32> = Some(u32::arbitrary(generator));
        let inline_end: Option<u32> = Some(u32::arbitrary(generator));

        FloatBandWrapper(FloatBand {
            top: Au::from_f32_px(top as f32),
            inline_start: inline_start.map(|value| Au::from_f32_px(value as f32)),
            inline_end: inline_end.map(|value| Au::from_f32_px(value as f32)),
        })
    }
}

#[derive(Clone, Debug)]
struct FloatRangeInput {
    start_index: u32,
    side: FloatSide,
    length: u32,
}

impl Arbitrary for FloatRangeInput {
    fn arbitrary(generator: &mut Gen) -> FloatRangeInput {
        let start_index: u32 = Arbitrary::arbitrary(generator);
        let is_left: bool = Arbitrary::arbitrary(generator);
        let length: u32 = Arbitrary::arbitrary(generator);
        FloatRangeInput {
            start_index,
            side: if is_left {
                FloatSide::InlineStart
            } else {
                FloatSide::InlineEnd
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

fn check_tree_find(tree: &FloatBandTree, block_position: Au, sorted_bands: &[FloatBand]) {
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
    assert_eq!(found_band.inline_start, reference_band.inline_start);
    assert_eq!(found_band.inline_end, reference_band.inline_end);
}

fn check_tree_find_next(tree: &FloatBandTree, block_position: Au, sorted_bands: &[FloatBand]) {
    let found_band = tree
        .find_next(block_position)
        .expect("Couldn't find the band in the tree!");
    let reference_band_index = sorted_bands
        .iter()
        .position(|band| band.top > block_position)
        .expect("Couldn't find the reference band!");
    let reference_band = &sorted_bands[reference_band_index];
    assert_eq!(found_band.top, reference_band.top);
    assert_eq!(found_band.inline_start, reference_band.inline_start);
    assert_eq!(found_band.inline_end, reference_band.inline_end);
}

fn check_node_range_setting(
    node: &FloatBandNode,
    block_range: &Range<Au>,
    side: FloatSide,
    value: Au,
) {
    if node.band.top >= block_range.start && node.band.top < block_range.end {
        match side {
            FloatSide::InlineStart => assert!(node.band.inline_start.unwrap() >= value),
            FloatSide::InlineEnd => assert!(node.band.inline_end.unwrap() <= value),
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
    block_range: &Range<Au>,
    side: FloatSide,
    value: Au,
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
    let f: fn(Vec<FloatBandWrapper>, Vec<u16>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>, lookups: Vec<u16>) {
        let mut bands: Vec<FloatBand> = bands.into_iter().map(|band| band.0).collect();
        bands.push(FloatBand {
            top: Au::zero(),
            inline_start: None,
            inline_end: None,
        });
        bands.push(FloatBand {
            top: Au::from_f32_px(INFINITY),
            inline_start: None,
            inline_end: None,
        });
        let mut tree = FloatBandTree::new();
        for band in &bands {
            tree = tree.insert(*band);
        }
        bands.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());
        for lookup in lookups {
            check_tree_find(&tree, Au::from_f32_px(lookup as f32), &bands);
        }
    }
}

// Tests that the `find_next()` method works.
#[test]
fn test_tree_find_next() {
    let f: fn(Vec<FloatBandWrapper>, Vec<u16>) = check;
    quickcheck::quickcheck(f);
    fn check(bands: Vec<FloatBandWrapper>, lookups: Vec<u16>) {
        let mut bands: Vec<FloatBand> = bands.into_iter().map(|band| band.0).collect();
        bands.push(FloatBand {
            top: Au::zero(),
            inline_start: None,
            inline_end: None,
        });
        bands.push(FloatBand {
            top: Au::from_f32_px(INFINITY),
            inline_start: None,
            inline_end: None,
        });
        bands.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());
        bands.dedup_by(|a, b| a.top == b.top);
        let mut tree = FloatBandTree::new();
        for band in &bands {
            tree = tree.insert(*band);
        }
        for lookup in lookups {
            check_tree_find_next(&tree, Au::from_f32_px(lookup as f32), &bands);
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
        for FloatBandWrapper(band) in &bands {
            tree = tree.insert(*band);
        }

        let mut tops: Vec<Au> = bands.iter().map(|band| band.0.top).collect();
        tops.push(Au::from_f32_px(INFINITY));
        tops.sort_by(|a, b| a.to_px().partial_cmp(&b.to_px()).unwrap());

        for range in ranges {
            let start = range.start_index.min(tops.len() as u32 - 1);
            let end = (range.start_index as u64 + range.length as u64).min(tops.len() as u64 - 1);
            let block_range = tops[start as usize]..tops[end as usize];
            let length = Au::from_px(range.length as i32);
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
    ceiling: Au,
    /// Containing block positioning information, which is used to track the current offsets
    /// from the float containing block formatting context to the current containing block.
    containing_block_info: ContainingBlockPositionInfo,
}

impl Arbitrary for FloatInput {
    fn arbitrary(generator: &mut Gen) -> FloatInput {
        // See #29819: Limit the maximum size of all f32 values here because
        // massive float values will start to introduce very bad floating point
        // errors.
        // TODO: This should be be addressed in a better way. Perhaps we should
        // reintroduce the use of app_units in Layout 2020.
        let width = u32::arbitrary(generator) % 12345;
        let height = u32::arbitrary(generator) % 12345;
        let is_left = bool::arbitrary(generator);
        let ceiling = u32::arbitrary(generator) % 12345;
        let left = u32::arbitrary(generator) % 12345;
        let containing_block_width = u32::arbitrary(generator) % 12345;
        let clear = u8::arbitrary(generator);
        FloatInput {
            info: PlacementInfo {
                size: LogicalVec2 {
                    inline: Au::from_f32_px(width as f32),
                    block: Au::from_f32_px(height as f32),
                },
                side: if is_left {
                    FloatSide::InlineStart
                } else {
                    FloatSide::InlineEnd
                },
                clear: new_clear(clear),
            },
            ceiling: Au::from_f32_px(ceiling as f32),
            containing_block_info: ContainingBlockPositionInfo::new_with_inline_offsets(
                Au::from_f32_px(left as f32),
                Au::from_f32_px(left as f32 + containing_block_width as f32),
            ),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = FloatInput>> {
        let mut this = (*self).clone();
        let mut shrunk = false;
        if let Some(inline_size) = self.info.size.inline.to_px().shrink().next() {
            this.info.size.inline = Au::from_px(inline_size);
            shrunk = true;
        }
        if let Some(block_size) = self.info.size.block.to_px().shrink().next() {
            this.info.size.block = Au::from_px(block_size);
            shrunk = true;
        }
        if let Some(clear) = (self.info.clear as u8).shrink().next() {
            this.info.clear = new_clear(clear);
            shrunk = true;
        }
        if let Some(left) = self
            .containing_block_info
            .inline_start
            .to_px()
            .shrink()
            .next()
        {
            this.containing_block_info.inline_start = Au::from_px(left);
            shrunk = true;
        }
        if let Some(right) = self
            .containing_block_info
            .inline_end
            .to_px()
            .shrink()
            .next()
        {
            this.containing_block_info.inline_end = Au::from_px(right);
            shrunk = true;
        }
        if let Some(ceiling) = self.ceiling.to_px().shrink().next() {
            this.ceiling = Au::from_px(ceiling);
            shrunk = true;
        }
        if shrunk {
            quickcheck::single_shrinker(this)
        } else {
            quickcheck::empty_shrinker()
        }
    }
}

fn new_clear(value: u8) -> Clear {
    match value & 3 {
        0 => Clear::None,
        1 => Clear::InlineStart,
        2 => Clear::InlineEnd,
        _ => Clear::Both,
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
    origin: LogicalVec2<Au>,
    info: PlacementInfo,
    ceiling: Au,
    containing_block_info: ContainingBlockPositionInfo,
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
                placed_float.containing_block_info.inline_start,
                placed_float.containing_block_info.inline_end,
            );
        }
        eprintln!("Bands:\n{:?}\n", self.float_context.bands);
    }
}

impl PlacedFloat {
    fn rect(&self) -> LogicalRect<Au> {
        LogicalRect {
            start_corner: self.origin,
            size: self.info.size,
        }
    }
}

impl FloatPlacement {
    fn place(floats: Vec<FloatInput>) -> FloatPlacement {
        let mut float_context = FloatContext::new(Au::from_f32_px(INFINITY));
        let mut placed_floats = vec![];
        for float in floats {
            let ceiling = float.ceiling;
            float_context.set_ceiling_from_non_floats(ceiling);
            float_context.containing_block_info = float.containing_block_info;
            placed_floats.push(PlacedFloat {
                origin: float_context.add_float(&float.info),
                info: float.info,
                ceiling,
                containing_block_info: float.containing_block_info,
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
            FloatSide::InlineStart => assert!(
                placed_float.origin.inline >= placed_float.containing_block_info.inline_start
            ),
            FloatSide::InlineEnd => {
                assert!(
                    placed_float.rect().max_inline_position() <=
                        placed_float.containing_block_info.inline_end
                )
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
                (FloatSide::InlineStart, FloatSide::InlineStart) => {
                    assert!(
                        this_float.origin.inline >= prev_float.rect().max_inline_position() ||
                            this_float.origin.block >= prev_float.rect().max_block_position()
                    );
                },
                (FloatSide::InlineEnd, FloatSide::InlineEnd) => {
                    assert!(
                        this_float.rect().max_inline_position() <= prev_float.origin.inline ||
                            this_float.origin.block >= prev_float.rect().max_block_position()
                    );
                },
                (FloatSide::InlineStart, FloatSide::InlineEnd) |
                (FloatSide::InlineEnd, FloatSide::InlineStart) => {},
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
                (this_float.info.size.block.is_zero() &&
                    this_float.rect().max_block_position() < other_float.origin.block) ||
                (this_float.info.size.block > Au::zero() &&
                    this_float.rect().max_block_position() <= other_float.origin.block)
            {
                continue;
            }

            match (this_float.info.side, other_float.info.side) {
                (FloatSide::InlineStart, FloatSide::InlineEnd) => {
                    assert!(this_float.rect().max_inline_position() <= other_float.origin.inline);
                },
                (FloatSide::InlineEnd, FloatSide::InlineStart) => {
                    assert!(this_float.origin.inline >= other_float.rect().max_inline_position());
                },
                (FloatSide::InlineStart, FloatSide::InlineStart) |
                (FloatSide::InlineEnd, FloatSide::InlineEnd) => {},
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
        assert!(placed_float.origin.block >= Au::zero());
    }
}

// 5. The outer top of a floating box may not be higher than the outer top of any block or floated
//    box generated by an element earlier in the source document.
fn check_floats_rule_5(placement: &FloatPlacement) {
    let mut block_position = Au::zero();
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
            FloatSide::InlineStart => {
                if placed_float.rect().max_inline_position() <=
                    placed_float.containing_block_info.inline_end
                {
                    continue;
                }
            },
            FloatSide::InlineEnd => {
                if placed_float.origin.inline >= placed_float.containing_block_info.inline_start {
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
        .map(|(float, _)| (*float).clone())
        .collect();
    let placement = FloatPlacement::place(floats);

    for (float_index, &(_, perturbation)) in floats_and_perturbations.iter().enumerate() {
        if perturbation == 0 {
            continue;
        }

        let mut placement = placement.clone();
        placement.placed_floats[float_index].origin.block -= Au::from_f32_px(perturbation as f32);

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
        .map(|(float, _)| (*float).clone())
        .collect();
    let placement = FloatPlacement::place(floats);

    for (float_index, &(_, perturbation)) in floats_and_perturbations.iter().enumerate() {
        if perturbation == 0 {
            continue;
        }

        let mut placement = placement.clone();
        {
            let placed_float = &mut placement.placed_floats[float_index];
            let perturbation = Au::from_f32_px(perturbation as f32);
            match placed_float.info.side {
                FloatSide::InlineStart => placed_float.origin.inline -= perturbation,
                FloatSide::InlineEnd => placed_float.origin.inline += perturbation,
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
    let mut block_position = Au::zero();
    for placed_float in &placement.placed_floats {
        assert!(placed_float.origin.block >= block_position);
        block_position = placed_float.origin.block;
    }

    for (this_float_index, this_float) in placement.placed_floats.iter().enumerate() {
        if this_float.info.clear == Clear::None {
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
                (this_float.info.size.block.is_zero() &&
                    this_float.rect().max_block_position() < other_float.origin.block) ||
                (this_float.info.size.block > Au::zero() &&
                    this_float.rect().max_block_position() <= other_float.origin.block)
            {
                continue;
            }

            match this_float.info.clear {
                Clear::InlineStart => assert_ne!(other_float.info.side, FloatSide::InlineStart),
                Clear::InlineEnd => assert_ne!(other_float.info.side, FloatSide::InlineEnd),
                Clear::Both => assert!(false),
                Clear::None => unreachable!(),
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
