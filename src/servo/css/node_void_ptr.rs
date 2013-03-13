//! CSS library requires that DOM nodes be convertable to *c_void through this trait

use dom::node::AbstractNode;

use core::cast;

// FIXME: Rust #3908. rust-css can't reexport VoidPtrLike
extern mod netsurfcss;
use css::node_void_ptr::netsurfcss::util::VoidPtrLike;

impl VoidPtrLike for AbstractNode {
    static fn from_void_ptr(node: *libc::c_void) -> AbstractNode {
        fail_unless!(node.is_not_null());
        unsafe {
            cast::transmute(node)
        }
    }

    fn to_void_ptr(&self) -> *libc::c_void {
        unsafe {
            cast::transmute(*self)
        }
    }
}
