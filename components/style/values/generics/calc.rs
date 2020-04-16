/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use crate::Zero;
use smallvec::SmallVec;
use std::fmt::{self, Write};
use std::ops::Add;
use std::{cmp, mem};
use style_traits::{CssWriter, ToCss};

/// Whether we're a `min` or `max` function.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    ToAnimatedZero,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum MinMaxOp {
    /// `min()`
    Min,
    /// `max()`
    Max,
}

/// This determines the order in which we serialize members of a calc() sum.
///
/// See https://drafts.csswg.org/css-values-4/#sort-a-calculations-children
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub enum SortKey {
    Number,
    Percentage,
    Ch,
    Deg,
    Em,
    Ex,
    Px,
    Rem,
    Sec,
    Vh,
    Vmax,
    Vmin,
    Vw,
    Other,
}

/// A generic node in a calc expression.
///
/// FIXME: This would be much more elegant if we used `Self` in the types below,
/// but we can't because of https://github.com/serde-rs/serde/issues/1565.
#[repr(u8)]
#[derive(
    Clone,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    ToAnimatedZero,
    ToResolvedValue,
    ToShmem,
)]
pub enum GenericCalcNode<L> {
    /// A leaf node.
    Leaf(L),
    /// A sum node, representing `a + b + c` where a, b, and c are the
    /// arguments.
    Sum(crate::OwnedSlice<GenericCalcNode<L>>),
    /// A `min` or `max` function.
    MinMax(crate::OwnedSlice<GenericCalcNode<L>>, MinMaxOp),
    /// A `clamp()` function.
    Clamp {
        /// The minimum value.
        min: Box<GenericCalcNode<L>>,
        /// The central value.
        center: Box<GenericCalcNode<L>>,
        /// The maximum value.
        max: Box<GenericCalcNode<L>>,
    },
}

pub use self::GenericCalcNode as CalcNode;

/// A trait that represents all the stuff a valid leaf of a calc expression.
pub trait CalcNodeLeaf: Clone + Sized + PartialOrd + PartialEq + ToCss {
    /// Whether this value is known-negative.
    fn is_negative(&self) -> bool;

    /// Tries to merge one sum to another, that is, perform `x` + `y`.
    fn try_sum_in_place(&mut self, other: &Self) -> Result<(), ()>;

    /// Multiplies the leaf by a given scalar number.
    fn mul_by(&mut self, scalar: f32);

    /// Negates the leaf.
    fn negate(&mut self) {
        self.mul_by(-1.);
    }

    /// Canonicalizes the expression if necessary.
    fn simplify(&mut self);

    /// Returns the sort key for simplification.
    fn sort_key(&self) -> SortKey;
}

impl<L: CalcNodeLeaf> CalcNode<L> {
    /// Negates the node.
    pub fn negate(&mut self) {
        self.mul_by(-1.);
    }

    fn sort_key(&self) -> SortKey {
        match *self {
            Self::Leaf(ref l) => l.sort_key(),
            _ => SortKey::Other,
        }
    }

    /// Tries to merge one sum to another, that is, perform `x` + `y`.
    fn try_sum_in_place(&mut self, other: &Self) -> Result<(), ()> {
        match (self, other) {
            (&mut CalcNode::Leaf(ref mut one), &CalcNode::Leaf(ref other)) => {
                one.try_sum_in_place(other)
            },
            _ => Err(()),
        }
    }

    /// Convert this `CalcNode` into a `CalcNode` with a different leaf kind.
    pub fn map_leaves<O, F>(&self, mut map: F) -> CalcNode<O>
    where
        O: CalcNodeLeaf,
        F: FnMut(&L) -> O,
    {
        self.map_leaves_internal(&mut map)
    }

    fn map_leaves_internal<O, F>(&self, map: &mut F) -> CalcNode<O>
    where
        O: CalcNodeLeaf,
        F: FnMut(&L) -> O,
    {
        fn map_children<L, O, F>(
            children: &[CalcNode<L>],
            map: &mut F,
        ) -> crate::OwnedSlice<CalcNode<O>>
        where
            L: CalcNodeLeaf,
            O: CalcNodeLeaf,
            F: FnMut(&L) -> O,
        {
            children
                .iter()
                .map(|c| c.map_leaves_internal(map))
                .collect()
        }

        match *self {
            Self::Leaf(ref l) => CalcNode::Leaf(map(l)),
            Self::Sum(ref c) => CalcNode::Sum(map_children(c, map)),
            Self::MinMax(ref c, op) => CalcNode::MinMax(map_children(c, map), op),
            Self::Clamp {
                ref min,
                ref center,
                ref max,
            } => {
                let min = Box::new(min.map_leaves_internal(map));
                let center = Box::new(center.map_leaves_internal(map));
                let max = Box::new(max.map_leaves_internal(map));
                CalcNode::Clamp { min, center, max }
            },
        }
    }

    /// Resolves the expression returning a value of `O`, given a function to
    /// turn a leaf into the relevant value.
    pub fn resolve<O>(
        &self,
        mut leaf_to_output_fn: impl FnMut(&L) -> Result<O, ()>,
    ) -> Result<O, ()>
    where
        O: PartialOrd + PartialEq + Add<Output = O> + Zero,
    {
        self.resolve_internal(&mut leaf_to_output_fn)
    }

    fn resolve_internal<O, F>(&self, leaf_to_output_fn: &mut F) -> Result<O, ()>
    where
        O: PartialOrd + PartialEq + Add<Output = O> + Zero,
        F: FnMut(&L) -> Result<O, ()>,
    {
        Ok(match *self {
            Self::Leaf(ref l) => return leaf_to_output_fn(l),
            Self::Sum(ref c) => {
                let mut result = Zero::zero();
                for child in &**c {
                    result = result + child.resolve_internal(leaf_to_output_fn)?;
                }
                result
            },
            Self::MinMax(ref nodes, op) => {
                let mut result = nodes[0].resolve_internal(leaf_to_output_fn)?;
                for node in nodes.iter().skip(1) {
                    let candidate = node.resolve_internal(leaf_to_output_fn)?;
                    let candidate_wins = match op {
                        MinMaxOp::Min => candidate < result,
                        MinMaxOp::Max => candidate > result,
                    };
                    if candidate_wins {
                        result = candidate;
                    }
                }
                result
            },
            Self::Clamp {
                ref min,
                ref center,
                ref max,
            } => {
                let min = min.resolve_internal(leaf_to_output_fn)?;
                let center = center.resolve_internal(leaf_to_output_fn)?;
                let max = max.resolve_internal(leaf_to_output_fn)?;

                let mut result = center;
                if result > max {
                    result = max;
                }
                if result < min {
                    result = min
                }
                result
            },
        })
    }

    fn is_negative_leaf(&self) -> bool {
        match *self {
            Self::Leaf(ref l) => l.is_negative(),
            _ => false,
        }
    }

    /// Multiplies the node by a scalar.
    pub fn mul_by(&mut self, scalar: f32) {
        match *self {
            Self::Leaf(ref mut l) => l.mul_by(scalar),
            // Multiplication is distributive across this.
            Self::Sum(ref mut children) => {
                for node in &mut **children {
                    node.mul_by(scalar);
                }
            },
            // This one is a bit trickier.
            Self::MinMax(ref mut children, ref mut op) => {
                for node in &mut **children {
                    node.mul_by(scalar);
                }

                // For negatives we need to invert the operation.
                if scalar < 0. {
                    *op = match *op {
                        MinMaxOp::Min => MinMaxOp::Max,
                        MinMaxOp::Max => MinMaxOp::Min,
                    }
                }
            },
            // This one is slightly tricky too.
            Self::Clamp {
                ref mut min,
                ref mut center,
                ref mut max,
            } => {
                min.mul_by(scalar);
                center.mul_by(scalar);
                max.mul_by(scalar);
                // For negatives we need to swap min / max.
                if scalar < 0. {
                    mem::swap(min, max);
                }
            },
        }
    }

    /// Visits all the nodes in this calculation tree recursively, starting by
    /// the leaves and bubbling all the way up.
    ///
    /// This is useful for simplification, but can also be used for validation
    /// and such.
    pub fn visit_depth_first(&mut self, mut f: impl FnMut(&mut Self)) {
        self.visit_depth_first_internal(&mut f);
    }

    fn visit_depth_first_internal(&mut self, f: &mut impl FnMut(&mut Self)) {
        match *self {
            Self::Clamp {
                ref mut min,
                ref mut center,
                ref mut max,
            } => {
                min.visit_depth_first_internal(f);
                center.visit_depth_first_internal(f);
                max.visit_depth_first_internal(f);
            },
            Self::Sum(ref mut children) | Self::MinMax(ref mut children, _) => {
                for child in &mut **children {
                    child.visit_depth_first_internal(f);
                }
            },
            Self::Leaf(..) => {},
        }
        f(self);
    }

    /// Simplifies and sorts the calculation of a given node. All the nodes
    /// below it should be simplified already, this only takes care of
    /// simplifying directly nested nodes. So, probably should always be used in
    /// combination with `visit_depth_first()`.
    ///
    /// This is only needed if it's going to be preserved after parsing (so, for
    /// `<length-percentage>`). Otherwise we can just evaluate it using
    /// `resolve()`, and we'll come up with a simplified value anyways.
    pub fn simplify_and_sort_direct_children(&mut self) {
        macro_rules! replace_self_with {
            ($slot:expr) => {{
                let dummy = Self::MinMax(Default::default(), MinMaxOp::Max);
                let result = mem::replace($slot, dummy);
                mem::replace(self, result);
            }};
        }
        match *self {
            Self::Clamp {
                ref mut min,
                ref mut center,
                ref mut max,
            } => {
                // NOTE: clamp() is max(min, min(center, max))
                let min_cmp_center = match min.partial_cmp(&center) {
                    Some(o) => o,
                    None => return,
                };

                // So if we can prove that min is more than center, then we won,
                // as that's what we should always return.
                if matches!(min_cmp_center, cmp::Ordering::Greater) {
                    return replace_self_with!(&mut **min);
                }

                // Otherwise try with max.
                let max_cmp_center = match max.partial_cmp(&center) {
                    Some(o) => o,
                    None => return,
                };

                if matches!(max_cmp_center, cmp::Ordering::Less) {
                    // max is less than center, so we need to return effectively
                    // `max(min, max)`.
                    let max_cmp_min = match max.partial_cmp(&min) {
                        Some(o) => o,
                        None => {
                            debug_assert!(
                                false,
                                "We compared center with min and max, how are \
                                 min / max not comparable with each other?"
                            );
                            return;
                        },
                    };

                    if matches!(max_cmp_min, cmp::Ordering::Less) {
                        return replace_self_with!(&mut **min);
                    }

                    return replace_self_with!(&mut **max);
                }

                // Otherwise we're the center node.
                return replace_self_with!(&mut **center);
            },
            Self::MinMax(ref mut children, op) => {
                let winning_order = match op {
                    MinMaxOp::Min => cmp::Ordering::Less,
                    MinMaxOp::Max => cmp::Ordering::Greater,
                };

                let mut result = 0;
                for i in 1..children.len() {
                    let o = match children[i].partial_cmp(&children[result]) {
                        // We can't compare all the children, so we can't
                        // know which one will actually win. Bail out and
                        // keep ourselves as a min / max function.
                        //
                        // TODO: Maybe we could simplify compatible children,
                        // see https://github.com/w3c/csswg-drafts/issues/4756
                        None => return,
                        Some(o) => o,
                    };

                    if o == winning_order {
                        result = i;
                    }
                }

                replace_self_with!(&mut children[result]);
            },
            Self::Sum(ref mut children_slot) => {
                let mut sums_to_merge = SmallVec::<[_; 3]>::new();
                let mut extra_kids = 0;
                for (i, child) in children_slot.iter().enumerate() {
                    if let Self::Sum(ref children) = *child {
                        extra_kids += children.len();
                        sums_to_merge.push(i);
                    }
                }

                // If we only have one kid, we've already simplified it, and it
                // doesn't really matter whether it's a sum already or not, so
                // lift it up and continue.
                if children_slot.len() == 1 {
                    return replace_self_with!(&mut children_slot[0]);
                }

                let mut children = mem::replace(children_slot, Default::default()).into_vec();

                if !sums_to_merge.is_empty() {
                    children.reserve(extra_kids - sums_to_merge.len());
                    // Merge all our nested sums, in reverse order so that the
                    // list indices are not invalidated.
                    for i in sums_to_merge.drain(..).rev() {
                        let kid_children = match children.swap_remove(i) {
                            Self::Sum(c) => c,
                            _ => unreachable!(),
                        };

                        // This would be nicer with
                        // https://github.com/rust-lang/rust/issues/59878 fixed.
                        children.extend(kid_children.into_vec());
                    }
                }

                debug_assert!(children.len() >= 2, "Should still have multiple kids!");

                // Sort by spec order.
                children.sort_unstable_by_key(|c| c.sort_key());

                // NOTE: if the function returns true, by the docs of dedup_by,
                // a is removed.
                children.dedup_by(|a, b| b.try_sum_in_place(a).is_ok());

                if children.len() == 1 {
                    // If only one children remains, lift it up, and carry on.
                    replace_self_with!(&mut children[0]);
                } else {
                    // Else put our simplified children back.
                    mem::replace(children_slot, children.into_boxed_slice().into());
                }
            },
            Self::Leaf(ref mut l) => {
                l.simplify();
            },
        }
    }

    /// Simplifies and sorts the kids in the whole calculation subtree.
    pub fn simplify_and_sort(&mut self) {
        self.visit_depth_first(|node| node.simplify_and_sort_direct_children())
    }

    fn to_css_impl<W>(&self, dest: &mut CssWriter<W>, is_outermost: bool) -> fmt::Result
    where
        W: Write,
    {
        let write_closing_paren = match *self {
            Self::MinMax(_, op) => {
                dest.write_str(match op {
                    MinMaxOp::Max => "max(",
                    MinMaxOp::Min => "min(",
                })?;
                true
            },
            Self::Clamp { .. } => {
                dest.write_str("clamp(")?;
                true
            },
            _ => {
                if is_outermost {
                    dest.write_str("calc(")?;
                }
                is_outermost
            },
        };

        match *self {
            Self::MinMax(ref children, _) => {
                let mut first = true;
                for child in &**children {
                    if !first {
                        dest.write_str(", ")?;
                    }
                    first = false;
                    child.to_css_impl(dest, false)?;
                }
            },
            Self::Sum(ref children) => {
                let mut first = true;
                for child in &**children {
                    if !first {
                        if child.is_negative_leaf() {
                            dest.write_str(" - ")?;
                            let mut c = child.clone();
                            c.negate();
                            c.to_css_impl(dest, false)?;
                        } else {
                            dest.write_str(" + ")?;
                            child.to_css_impl(dest, false)?;
                        }
                    } else {
                        first = false;
                        child.to_css_impl(dest, false)?;
                    }
                }
            },
            Self::Clamp {
                ref min,
                ref center,
                ref max,
            } => {
                min.to_css_impl(dest, false)?;
                dest.write_str(", ")?;
                center.to_css_impl(dest, false)?;
                dest.write_str(", ")?;
                max.to_css_impl(dest, false)?;
            },
            Self::Leaf(ref l) => l.to_css(dest)?,
        }

        if write_closing_paren {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl<L: CalcNodeLeaf> PartialOrd for CalcNode<L> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (&CalcNode::Leaf(ref one), &CalcNode::Leaf(ref other)) => one.partial_cmp(other),
            _ => None,
        }
    }
}

impl<L: CalcNodeLeaf> ToCss for CalcNode<L> {
    /// <https://drafts.csswg.org/css-values/#calc-serialize>
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.to_css_impl(dest, /* is_outermost = */ true)
    }
}
