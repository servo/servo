import spidermonkey::jsapi::{JSContext, jsval};

impl methods<T: copy> for *T {
    unsafe fn +(idx: uint) -> *T {
        ptr::offset(self, idx)
    }
    unsafe fn [](idx: uint) -> T {
        *(self + idx)
    }
}

const JSVAL_VOID: u64 =  0x0001fff2_00000000_u64;
const JSVAL_NULL: u64 =  0x0001fff6_00000000_u64;
const JSVAL_ZERO: u64 =  0x0001fff1_00000000_u64;
const JSVAL_ONE: u64 =   0x0001fff1_00000001_u64;
const JSVAL_FALSE: u64 = 0x0001fff3_00000000_u64;
const JSVAL_TRUE: u64 =  0x0001fff3_00000001_u64;

unsafe fn JS_ARGV(_cx: *JSContext, vp: *jsval) -> *jsval {
    vp + 2u
}

unsafe fn JS_SET_RVAL(_cx: *JSContext, vp: *jsval, v: jsval) {
    let vp: *mut jsval = unsafe::reinterpret_cast(vp);
    *vp = v;
}

