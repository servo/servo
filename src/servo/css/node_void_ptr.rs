//! CSS library requires that DOM nodes be convertable to *c_void through this trait

use dom::node::Node;

// FIXME: Rust #3908. rust-css can't reexport VoidPtrLike
extern mod netsurfcss;
use css::node_void_ptr::netsurfcss::util::VoidPtrLike;

impl VoidPtrLike for Node {
    static fn from_void_ptr(node: *libc::c_void) -> Node {
        assert node.is_not_null();
        unsafe { cast::reinterpret_cast(&node) }
    }

    fn to_void_ptr(&self) -> *libc::c_void {
        let node: *libc::c_void = unsafe { cast::reinterpret_cast(self) };
        node
    }
}
