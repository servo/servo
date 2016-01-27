#![feature(plugin)]
#![plugin(plugins)]

extern crate js;

use std::cell::Cell;

use js::jsval::JSVal;


struct Foo {
    bar: Cell<JSVal>
    //~^ ERROR Banned type Cell<JSVal> detected. Use MutHeap<JSVal> instead,
}

fn main() {}
