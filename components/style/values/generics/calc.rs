/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use num_traits::{Float, Zero};
use smallvec::SmallVec;
use std::fmt::{self, Write};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
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

/// Whether we're a `mod` or `rem` function.
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
pub enum ModRemOp {
    /// `mod()`
    Mod,
    /// `rem()`
    Rem,
}

/// The strategy used in `round()`
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
pub enum RoundingStrategy {
    /// `round(nearest, a, b)`
    /// round a to the nearest multiple of b
    Nearest,
    /// `round(up, a, b)`
    /// round a up to the nearest multiple of b
    Up,
    /// `round(down, a, b)`
    /// round a down to the nearest multiple of b
    Down,
    /// `round(to-zero, a, b)`
    /// round a to the nearest multiple of b that is towards zero
    ToZero,
}

/// This determines the order in which we serialize members of a calc() sum.
///
/// See https://drafts.csswg.org/css-values-4/#sort-a-calculations-children
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub enum SortKey {
    Number,
    Percentage,
    Cap,
    Ch,
    Cqb,
    Cqh,
    Cqi,
    Cqmax,
    Cqmin,
    Cqw,
    Deg,
    Dppx,
    Dvb,
    Dvh,
    Dvi,
    Dvmax,
    Dvmin,
    Dvw,
    Em,
    Ex,
    Ic,
    Lvb,
    Lvh,
    Lvi,
    Lvmax,
    Lvmin,
    Lvw,
    Px,
    Rem,
    Sec,
    Svb,
    Svh,
    Svi,
    Svmax,
    Svmin,
    Svw,
    Vb,
    Vh,
    Vi,
    Vmax,
    Vmin,
    Vw,
    Other,
}

/// A generic node in a calc expression.
///
/// FIXME: This would be much more elegant if we used `Self` in the types below,
/// but we can't because of https://github.com/serde-rs/serde/issues/1565.
///
/// FIXME: The following annotations are to workaround an LLVM inlining bug, see
/// bug 1631929.
///
/// cbindgen:destructor-attributes=MOZ_NEVER_INLINE
/// cbindgen:copy-constructor-attributes=MOZ_NEVER_INLINE
/// cbindgen:eq-attributes=MOZ_NEVER_INLINE
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
    /// A node that negates its children, e.g. Negate(1) == -1.
    Negate(Box<GenericCalcNode<L>>),
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
    /// A `round()` function.
    Round {
        /// The rounding strategy.
        strategy: RoundingStrategy,
        /// The value to round.
        value: Box<GenericCalcNode<L>>,
        /// The step value.
        step: Box<GenericCalcNode<L>>,
    },
    /// A `mod()` or `rem()` function.
    ModRem {
        /// The dividend calculation.
        dividend: Box<GenericCalcNode<L>>,
        /// The divisor calculation.
        divisor: Box<GenericCalcNode<L>>,
        /// Is the function mod or rem?
        op: ModRemOp,
    },
    /// A `hypot()` function
    Hypot(crate::OwnedSlice<GenericCalcNode<L>>),
}

pub use self::GenericCalcNode as CalcNode;

/// A trait that represents all the stuff a valid leaf of a calc expression.
pub trait CalcNodeLeaf: Clone + Sized + PartialOrd + PartialEq + ToCss {
    /// Returns the unitless value of this leaf.
    fn unitless_value(&self) -> f32;

    /// Whether this value is known-negative.
    fn is_negative(&self) -> bool {
        self.unitless_value().is_sign_negative()
    }

    /// Whether this value is infinite.
    fn is_infinite(&self) -> bool {
        self.unitless_value().is_infinite()
    }

    /// Whether this value is zero.
    fn is_zero(&self) -> bool {
        self.unitless_value().is_zero()
    }

    /// Whether this value is NaN.
    fn is_nan(&self) -> bool {
        self.unitless_value().is_nan()
    }

    /// Tries to merge one sum to another, that is, perform `x` + `y`.
    fn try_sum_in_place(&mut self, other: &Self) -> Result<(), ()>;

    /// Tries a generic arithmetic operation.
    fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32;

    /// Map the value of this node with the given operation.
    fn map(&mut self, op: impl FnMut(f32) -> f32);

    /// Negates the leaf.
    fn negate(&mut self) {
        self.map(std::ops::Neg::neg);
    }

    /// Canonicalizes the expression if necessary.
    fn simplify(&mut self);

    /// Returns the sort key for simplification.
    fn sort_key(&self) -> SortKey;
}

/// The level of any argument being serialized in `to_css_impl`.
enum ArgumentLevel {
    /// The root of a calculation tree.
    CalculationRoot,
    /// The root of an operand node's argument, e.g. `min(10, 20)`, `10` and `20` will have this
    /// level, but min in this case will have `TopMost`.
    ArgumentRoot,
    /// Any other values serialized in the tree.
    Nested,
}

impl<L: CalcNodeLeaf> CalcNode<L> {
    /// Negate the node inline.  If the node is distributive, it is replaced by the result,
    /// otherwise the node is wrapped in a [`Negate`] node.
    pub fn negate(&mut self) {
        match *self {
            CalcNode::Leaf(ref mut leaf) => leaf.map(|l| l.neg()),
            CalcNode::Negate(ref mut value) => {
                // Don't negate the value here.  Replace `self` with it's child.
                let result = mem::replace(
                    value.as_mut(),
                    Self::MinMax(Default::default(), MinMaxOp::Max),
                );
                *self = result;
            },
            CalcNode::Sum(ref mut children) => {
                for child in children.iter_mut() {
                    child.negate();
                }
            },
            CalcNode::MinMax(ref mut children, ref mut op) => {
                for child in children.iter_mut() {
                    child.negate();
                }

                // Negating min-max means the operation is swapped.
                *op = match *op {
                    MinMaxOp::Min => MinMaxOp::Max,
                    MinMaxOp::Max => MinMaxOp::Min,
                };
            },
            CalcNode::Clamp {
                ref mut min,
                ref mut center,
                ref mut max,
            } => {
                min.negate();
                center.negate();
                max.negate();

                mem::swap(min, max);
            },
            CalcNode::Round {
                ref mut value,
                ref mut step,
                ..
            } => {
                value.negate();
                step.negate();
            },
            CalcNode::ModRem {
                ref mut dividend,
                ref mut divisor,
                ..
            } => {
                dividend.negate();
                divisor.negate();
            },
            CalcNode::Hypot(ref mut children) => {
                for child in children.iter_mut() {
                    child.negate();
                }
            },
        }
    }

    fn sort_key(&self) -> SortKey {
        match *self {
            Self::Leaf(ref l) => l.sort_key(),
            _ => SortKey::Other,
        }
    }

    /// Returns the leaf if we can (if simplification has allowed it).
    pub fn as_leaf(&self) -> Option<&L> {
        match *self {
            Self::Leaf(ref l) => Some(l),
            _ => None,
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

    /// Tries to apply a generic arithmentic operator
    fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        match (self, other) {
            (&CalcNode::Leaf(ref one), &CalcNode::Leaf(ref other)) => {
                Ok(CalcNode::Leaf(one.try_op(other, op)?))
            },
            _ => Err(()),
        }
    }

    /// Map the value of this node with the given operation.
    pub fn map(&mut self, mut op: impl FnMut(f32) -> f32) {
        fn map_internal<L: CalcNodeLeaf>(node: &mut CalcNode<L>, op: &mut impl FnMut(f32) -> f32) {
            match node {
                CalcNode::Leaf(l) => l.map(op),
                CalcNode::Negate(v) => map_internal(v, op),
                CalcNode::Sum(children) => {
                    for node in &mut **children {
                        map_internal(node, op);
                    }
                },
                CalcNode::MinMax(children, _) => {
                    for node in &mut **children {
                        map_internal(node, op);
                    }
                },
                CalcNode::Clamp { min, center, max } => {
                    map_internal(min, op);
                    map_internal(center, op);
                    map_internal(max, op);
                },
                CalcNode::Round { value, step, .. } => {
                    map_internal(value, op);
                    map_internal(step, op);
                },
                CalcNode::ModRem {
                    dividend, divisor, ..
                } => {
                    map_internal(dividend, op);
                    map_internal(divisor, op);
                },
                CalcNode::Hypot(children) => {
                    for node in &mut **children {
                        map_internal(node, op);
                    }
                },
            }
        }

        map_internal(self, &mut op);
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
            Self::Negate(ref c) => CalcNode::Negate(Box::new(c.map_leaves_internal(map))),
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
            Self::Round {
                strategy,
                ref value,
                ref step,
            } => {
                let value = Box::new(value.map_leaves_internal(map));
                let step = Box::new(step.map_leaves_internal(map));
                CalcNode::Round {
                    strategy,
                    value,
                    step,
                }
            },
            Self::ModRem {
                ref dividend,
                ref divisor,
                op,
            } => {
                let dividend = Box::new(dividend.map_leaves_internal(map));
                let divisor = Box::new(divisor.map_leaves_internal(map));
                CalcNode::ModRem {
                    dividend,
                    divisor,
                    op,
                }
            },
            Self::Hypot(ref c) => CalcNode::Hypot(map_children(c, map)),
        }
    }

    /// Resolves the expression returning a value of `O`, given a function to
    /// turn a leaf into the relevant value.
    pub fn resolve<O>(
        &self,
        mut leaf_to_output_fn: impl FnMut(&L) -> Result<O, ()>,
    ) -> Result<O, ()>
    where
        O: PartialOrd
            + PartialEq
            + Add<Output = O>
            + Mul<Output = O>
            + Div<Output = O>
            + Sub<Output = O>
            + Zero
            + Float
            + Copy,
    {
        self.resolve_internal(&mut leaf_to_output_fn)
    }

    fn resolve_internal<O, F>(&self, leaf_to_output_fn: &mut F) -> Result<O, ()>
    where
        O: PartialOrd
            + PartialEq
            + Add<Output = O>
            + Mul<Output = O>
            + Div<Output = O>
            + Sub<Output = O>
            + Zero
            + Float
            + Copy,
        F: FnMut(&L) -> Result<O, ()>,
    {
        Ok(match *self {
            Self::Leaf(ref l) => return leaf_to_output_fn(l),
            Self::Negate(ref c) => c.resolve_internal(leaf_to_output_fn)?.neg(),
            Self::Sum(ref c) => {
                let mut result = Zero::zero();
                for child in &**c {
                    result = result + child.resolve_internal(leaf_to_output_fn)?;
                }
                result
            },
            Self::MinMax(ref nodes, op) => {
                let mut result = nodes[0].resolve_internal(leaf_to_output_fn)?;

                if result.is_nan() {
                    return Ok(result);
                }

                for node in nodes.iter().skip(1) {
                    let candidate = node.resolve_internal(leaf_to_output_fn)?;

                    if candidate.is_nan() {
                        result = candidate;
                        break;
                    }

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

                if min.is_nan() || center.is_nan() || max.is_nan() {
                    result = <O as Float>::nan();
                }

                result
            },
            Self::Round {
                strategy,
                ref value,
                ref step,
            } => {
                let value = value.resolve_internal(leaf_to_output_fn)?;
                let step = step.resolve_internal(leaf_to_output_fn)?;

                // TODO(emilio): Seems like at least a few of these
                // special-cases could be removed if we do the math in a
                // particular order.
                if step.is_zero() {
                    return Ok(<O as Float>::nan());
                }

                if value.is_infinite() && step.is_infinite() {
                    return Ok(<O as Float>::nan());
                }

                if value.is_infinite() {
                    return Ok(value);
                }

                if step.is_infinite() {
                    match strategy {
                        RoundingStrategy::Nearest | RoundingStrategy::ToZero => {
                            return if value.is_sign_negative() {
                                Ok(<O as Float>::neg_zero())
                            } else {
                                Ok(<O as Zero>::zero())
                            }
                        },
                        RoundingStrategy::Up => {
                            return if !value.is_sign_negative() && !value.is_zero() {
                                Ok(<O as Float>::infinity())
                            } else if !value.is_sign_negative() && value.is_zero() {
                                Ok(value)
                            } else {
                                Ok(<O as Float>::neg_zero())
                            }
                        },
                        RoundingStrategy::Down => {
                            return if value.is_sign_negative() && !value.is_zero() {
                                Ok(<O as Float>::neg_infinity())
                            } else if value.is_sign_negative() && value.is_zero() {
                                Ok(value)
                            } else {
                                Ok(<O as Zero>::zero())
                            }
                        },
                    }
                }

                let div = value / step;
                let lower_bound = div.floor() * step;
                let upper_bound = div.ceil() * step;

                match strategy {
                    RoundingStrategy::Nearest => {
                        // In case of a tie, use the upper bound
                        if value - lower_bound < upper_bound - value {
                            lower_bound
                        } else {
                            upper_bound
                        }
                    },
                    RoundingStrategy::Up => upper_bound,
                    RoundingStrategy::Down => lower_bound,
                    RoundingStrategy::ToZero => {
                        // In case of a tie, use the upper bound
                        if lower_bound.abs() < upper_bound.abs() {
                            lower_bound
                        } else {
                            upper_bound
                        }
                    },
                }
            },
            Self::ModRem {
                ref dividend,
                ref divisor,
                op,
            } => {
                let dividend = dividend.resolve_internal(leaf_to_output_fn)?;
                let divisor = divisor.resolve_internal(leaf_to_output_fn)?;

                // In mod(A, B) only, if B is infinite and A has opposite sign to B
                // (including an oppositely-signed zero), the result is NaN.
                // https://drafts.csswg.org/css-values/#round-infinities
                if matches!(op, ModRemOp::Mod) &&
                    divisor.is_infinite() &&
                    dividend.is_sign_negative() != divisor.is_sign_negative()
                {
                    return Ok(<O as Float>::nan());
                }

                match op {
                    ModRemOp::Mod => dividend - divisor * (dividend / divisor).floor(),
                    ModRemOp::Rem => dividend - divisor * (dividend / divisor).trunc(),
                }
            },
            Self::Hypot(ref c) => {
                let mut result: O = Zero::zero();
                for child in &**c {
                    result = result + child.resolve_internal(leaf_to_output_fn)?.powi(2);
                }
                result.sqrt()
            },
        })
    }

    fn is_negative_leaf(&self) -> bool {
        match *self {
            Self::Leaf(ref l) => l.is_negative(),
            _ => false,
        }
    }

    fn is_zero_leaf(&self) -> bool {
        match *self {
            Self::Leaf(ref l) => l.is_zero(),
            _ => false,
        }
    }

    fn is_infinite_leaf(&self) -> bool {
        match *self {
            Self::Leaf(ref l) => l.is_infinite(),
            _ => false,
        }
    }

    /// Multiplies the node by a scalar.
    pub fn mul_by(&mut self, scalar: f32) {
        match *self {
            Self::Leaf(ref mut l) => l.map(|v| v * scalar),
            Self::Negate(ref mut value) => value.mul_by(scalar),
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
            Self::Round {
                ref mut value,
                ref mut step,
                ..
            } => {
                value.mul_by(scalar);
                step.mul_by(scalar);
            },
            Self::ModRem {
                ref mut dividend,
                ref mut divisor,
                ..
            } => {
                dividend.mul_by(scalar);
                divisor.mul_by(scalar);
            },
            // Not possible to handle negatives in this case, see: https://bugzil.la/1815448
            Self::Hypot(ref mut children) => {
                for node in &mut **children {
                    node.mul_by(scalar);
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
            Self::Round {
                ref mut value,
                ref mut step,
                ..
            } => {
                value.visit_depth_first_internal(f);
                step.visit_depth_first_internal(f);
            },
            Self::ModRem {
                ref mut dividend,
                ref mut divisor,
                ..
            } => {
                dividend.visit_depth_first_internal(f);
                divisor.visit_depth_first_internal(f);
            },
            Self::Sum(ref mut children) |
            Self::MinMax(ref mut children, _) |
            Self::Hypot(ref mut children) => {
                for child in &mut **children {
                    child.visit_depth_first_internal(f);
                }
            },
            Self::Negate(ref mut value) => {
                value.visit_depth_first_internal(f);
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
    ///
    /// <https://drafts.csswg.org/css-values-4/#calc-simplification>
    pub fn simplify_and_sort_direct_children(&mut self) {
        macro_rules! replace_self_with {
            ($slot:expr) => {{
                let dummy = Self::MinMax(Default::default(), MinMaxOp::Max);
                let result = mem::replace($slot, dummy);
                *self = result;
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
            Self::Round {
                strategy,
                ref mut value,
                ref mut step,
            } => {
                if step.is_zero_leaf() {
                    value.mul_by(f32::NAN);
                    return replace_self_with!(&mut **value);
                }

                if value.is_infinite_leaf() && step.is_infinite_leaf() {
                    value.mul_by(f32::NAN);
                    return replace_self_with!(&mut **value);
                }

                if value.is_infinite_leaf() {
                    return replace_self_with!(&mut **value);
                }

                if step.is_infinite_leaf() {
                    match strategy {
                        RoundingStrategy::Nearest | RoundingStrategy::ToZero => {
                            value.mul_by(0.);
                            return replace_self_with!(&mut **value);
                        },
                        RoundingStrategy::Up => {
                            if !value.is_negative_leaf() && !value.is_zero_leaf() {
                                value.mul_by(f32::INFINITY);
                                return replace_self_with!(&mut **value);
                            } else if !value.is_negative_leaf() && value.is_zero_leaf() {
                                return replace_self_with!(&mut **value);
                            } else {
                                value.mul_by(0.);
                                return replace_self_with!(&mut **value);
                            }
                        },
                        RoundingStrategy::Down => {
                            if value.is_negative_leaf() && !value.is_zero_leaf() {
                                value.mul_by(f32::INFINITY);
                                return replace_self_with!(&mut **value);
                            } else if value.is_negative_leaf() && value.is_zero_leaf() {
                                return replace_self_with!(&mut **value);
                            } else {
                                value.mul_by(0.);
                                return replace_self_with!(&mut **value);
                            }
                        },
                    }
                }

                if step.is_negative_leaf() {
                    step.negate();
                }

                let remainder = match value.try_op(step, Rem::rem) {
                    Ok(res) => res,
                    Err(..) => return,
                };

                let (mut lower_bound, mut upper_bound) = if value.is_negative_leaf() {
                    let upper_bound = match value.try_op(&remainder, Sub::sub) {
                        Ok(res) => res,
                        Err(..) => return,
                    };

                    let lower_bound = match upper_bound.try_op(&step, Sub::sub) {
                        Ok(res) => res,
                        Err(..) => return,
                    };

                    (lower_bound, upper_bound)
                } else {
                    let lower_bound = match value.try_op(&remainder, Sub::sub) {
                        Ok(res) => res,
                        Err(..) => return,
                    };

                    let upper_bound = match lower_bound.try_op(&step, Add::add) {
                        Ok(res) => res,
                        Err(..) => return,
                    };

                    (lower_bound, upper_bound)
                };

                match strategy {
                    RoundingStrategy::Nearest => {
                        let lower_diff = match value.try_op(&lower_bound, Sub::sub) {
                            Ok(res) => res,
                            Err(..) => return,
                        };

                        let upper_diff = match upper_bound.try_op(value, Sub::sub) {
                            Ok(res) => res,
                            Err(..) => return,
                        };

                        // In case of a tie, use the upper bound
                        if lower_diff < upper_diff {
                            return replace_self_with!(&mut lower_bound);
                        } else {
                            return replace_self_with!(&mut upper_bound);
                        }
                    },
                    RoundingStrategy::Up => return replace_self_with!(&mut upper_bound),
                    RoundingStrategy::Down => return replace_self_with!(&mut lower_bound),
                    RoundingStrategy::ToZero => {
                        let mut lower_diff = lower_bound.clone();
                        let mut upper_diff = upper_bound.clone();

                        if lower_diff.is_negative_leaf() {
                            lower_diff.negate();
                        }

                        if upper_diff.is_negative_leaf() {
                            upper_diff.negate();
                        }

                        // In case of a tie, use the upper bound
                        if lower_diff < upper_diff {
                            return replace_self_with!(&mut lower_bound);
                        } else {
                            return replace_self_with!(&mut upper_bound);
                        }
                    },
                };
            },
            Self::ModRem {
                ref dividend,
                ref divisor,
                op,
            } => {
                let mut result = dividend.clone();

                // In mod(A, B) only, if B is infinite and A has opposite sign to B
                // (including an oppositely-signed zero), the result is NaN.
                // https://drafts.csswg.org/css-values/#round-infinities
                if matches!(op, ModRemOp::Mod) &&
                    divisor.is_infinite_leaf() &&
                    dividend.is_negative_leaf() != divisor.is_negative_leaf()
                {
                    result.mul_by(f32::NAN);
                    return replace_self_with!(&mut *result);
                }

                let result = match op {
                    ModRemOp::Mod => dividend.try_op(divisor, |a, b| a - b * (a / b).floor()),
                    ModRemOp::Rem => dividend.try_op(divisor, |a, b| a - b * (a / b).trunc()),
                };

                let mut result = match result {
                    Ok(res) => res,
                    Err(..) => return,
                };

                return replace_self_with!(&mut result);
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
                    *children_slot = children.into_boxed_slice().into();
                }
            },
            Self::Hypot(ref children) => {
                let mut result = match children[0].try_op(&children[0], Mul::mul) {
                    Ok(res) => res,
                    Err(..) => return,
                };

                for child in children.iter().skip(1) {
                    let square = match child.try_op(&child, Mul::mul) {
                        Ok(res) => res,
                        Err(..) => return,
                    };
                    result = match result.try_op(&square, Add::add) {
                        Ok(res) => res,
                        Err(..) => return,
                    }
                }

                result = match result.try_op(&result, |a, _| a.sqrt()) {
                    Ok(res) => res,
                    Err(..) => return,
                };

                replace_self_with!(&mut result);
            },
            Self::Negate(ref mut child) => {
                // Step 6.
                match &mut **child {
                    CalcNode::Leaf(_) => {
                        // 1. If root’s child is a numeric value, return an equivalent numeric value, but
                        // with the value negated (0 - value).
                        child.negate();
                        replace_self_with!(&mut **child);
                    },
                    CalcNode::Negate(value) => {
                        // 2. If root’s child is a Negate node, return the child’s child.
                        replace_self_with!(&mut **value);
                    },
                    _ => {
                        // 3. Return root.
                    },
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

    fn to_css_impl<W>(&self, dest: &mut CssWriter<W>, level: ArgumentLevel) -> fmt::Result
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
            Self::Round { strategy, .. } => {
                match strategy {
                    RoundingStrategy::Nearest => dest.write_str("round("),
                    RoundingStrategy::Up => dest.write_str("round(up, "),
                    RoundingStrategy::Down => dest.write_str("round(down, "),
                    RoundingStrategy::ToZero => dest.write_str("round(to-zero, "),
                }?;

                true
            },
            Self::ModRem { op, .. } => {
                dest.write_str(match op {
                    ModRemOp::Mod => "mod(",
                    ModRemOp::Rem => "rem(",
                })?;

                true
            },
            Self::Hypot(_) => {
                dest.write_str("hypot(")?;
                true
            },
            Self::Negate(_) => {
                // We never generate a [`Negate`] node as the root of a calculation, only inside
                // [`Sum`] nodes as a child. Because negate nodes are handled by the [`Sum`] node
                // directly (see below), this node will never be serialized.
                debug_assert!(
                    false,
                    "We never serialize Negate nodes as they are handled inside Sum nodes."
                );
                dest.write_str("(-1 * ")?;
                true
            },
            Self::Sum(_) => match level {
                ArgumentLevel::CalculationRoot => {
                    dest.write_str("calc(")?;
                    true
                },
                ArgumentLevel::ArgumentRoot => false,
                ArgumentLevel::Nested => {
                    dest.write_str("(")?;
                    true
                },
            },
            Self::Leaf(_) => match level {
                ArgumentLevel::CalculationRoot => {
                    dest.write_str("calc(")?;
                    true
                },
                ArgumentLevel::ArgumentRoot | ArgumentLevel::Nested => false,
            },
        };

        match *self {
            Self::MinMax(ref children, _) | Self::Hypot(ref children) => {
                let mut first = true;
                for child in &**children {
                    if !first {
                        dest.write_str(", ")?;
                    }
                    first = false;
                    child.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
                }
            },
            Self::Negate(ref value) => value.to_css_impl(dest, ArgumentLevel::Nested)?,
            Self::Sum(ref children) => {
                let mut first = true;
                for child in &**children {
                    if !first {
                        match child {
                            Self::Leaf(l) => {
                                if l.is_negative() {
                                    dest.write_str(" - ")?;
                                    let mut negated = l.clone();
                                    negated.negate();
                                    negated.to_css(dest)?;
                                } else {
                                    dest.write_str(" + ")?;
                                    l.to_css(dest)?;
                                }
                            },
                            Self::Negate(n) => {
                                dest.write_str(" - ")?;
                                n.to_css_impl(dest, ArgumentLevel::Nested)?;
                            },
                            _ => {
                                dest.write_str(" + ")?;
                                child.to_css_impl(dest, ArgumentLevel::Nested)?;
                            },
                        }
                    } else {
                        first = false;
                        child.to_css_impl(dest, ArgumentLevel::Nested)?;
                    }
                }
            },
            Self::Clamp {
                ref min,
                ref center,
                ref max,
            } => {
                min.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
                dest.write_str(", ")?;
                center.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
                dest.write_str(", ")?;
                max.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
            },
            Self::Round {
                ref value,
                ref step,
                ..
            } => {
                value.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
                dest.write_str(", ")?;
                step.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
            },
            Self::ModRem {
                ref dividend,
                ref divisor,
                ..
            } => {
                dividend.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
                dest.write_str(", ")?;
                divisor.to_css_impl(dest, ArgumentLevel::ArgumentRoot)?;
            },
            Self::Leaf(ref l) => l.to_css(dest)?,
        }

        if write_closing_paren {
            dest.write_char(')')?;
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
        self.to_css_impl(dest, ArgumentLevel::CalculationRoot)
    }
}
