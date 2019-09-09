use super::*;

#[derive(Debug)]
pub(in crate::layout) struct FloatBox {
    pub style: Arc<ComputedValues>,
    pub contents: IndependentFormattingContext,
}

/// Data kept during layout about the floats in a given block formatting context.
pub(in crate::layout) struct FloatContext {
    // TODO
}

impl FloatContext {
    pub fn new() -> Self {
        FloatContext {}
    }
}
