use std::sync::Arc;

pub unsafe trait HasStrong {
    type Strong;
    fn into_strong(x: Arc<Self>) -> Self::Strong;
    fn null_strong() -> Self::Strong;
}