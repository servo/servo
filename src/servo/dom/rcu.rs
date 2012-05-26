#[doc(str = "

Implements the RCU dom-sharing model.  This model allows for a single
writer and any number of readers, but the writer must be able to
control and manage the lifetimes of the reader(s).  For simplicity I
will describe the impl as though there were a single reader.

The basic idea is that every object in the RCU pool has both a reader
view and a writer view.  The writer always sees the writer view, which
contains the most up-to-date values.  The reader uses the reader view,
which contains the values as of the point where the reader was forked.
When the writer joins the reader, the reader view will be synchronized
with the writer view.

Internally, the way this works is using a copy-on-write scheme.  Each
RCU node maintains two pointers (`rd_ptr` and `wr_ptr`).  Assuming
that readers are active, when a writer wants to modify a node, it
first copies the reader's data into a new pointer.  Any writes that
occur after that point (but before the reader is joined) will operate
on this same copy.  When the reader is joined, any nodes which the
writer modified will free the stale reader data and update the reader
pointer to be the same as the writer pointer.

# Using the RCU APIs as a writer

You must first create a `scope` object.  The scope object manages the
memory and the RCU operations.  RCU'd objects of some sendable type
`T` are not referenced directly but rather through a `handle<T>`.  To
create a new RCU object, you use `scope.handle(t)` where `t` is some
initial value of type `T`.  To write to an RCU object, use
`scope.wr()` and to read from it use `scope.rd()`. Be sure not to use
the various `reader_methods`.

Handles can be freely sent between tasks but the RCU scope cannot.  It
must stay with the writer task.  You are responsible for correctly
invoking `reader_forked()` and `reader_joined()` to keep the RCU scope
abreast of when the reader is active.  Failure to do so will lead to
race conditions or worse.

# Using the RCU APIs as a reader

Import the `reader_methods` impl.  When you receive a handle, you can
invoke `h.rd { |v| ... }` and so forth.  There is also a piece of
auxiliary data that can be optionally associated with each handle.

Note: if the type `T` contains mutable fields, then there is nothing
to stop the reader from mutating those fields in the `rd()` method.
Do not do this.  It will lead to race conditions.

# Auxiliary data

Readers can associate a piece of auxiliary data of type `A` along with
main nodes.  This is convenient but dangerous: it is the reader's job
to ensure that this data remains live independent of the RCU nodes
themselves.

")];

import ptr::extensions;

export handle;
export reader_methods;
export writer_methods;
export scope;

type scope_data<T:send,A> = {
    mut layout_active: bool,
    mut free_list: [handle<T,A>],
    mut first_dirty: handle<T,A>
};

resource scope_rsrc<T:send,A>(d: scope_data<T,A>) {
    unsafe {
        for d.free_list.each { |h| free_handle(h); }
    }
}

type scope<T:send,A> = @scope_rsrc<T,A>;

type handle_data<T:send,A> = {mut rd_ptr: *T,
                              mut wr_ptr: *T,
                              mut rd_aux: *A,
                              mut next_dirty: handle<T,A>};
enum handle<T:send,A> {
    _handle(*handle_data<T,A>)
}

impl private_methods<T:send,A> for handle<T,A> {
    fn rd_ptr() -> *T unsafe { (**self).rd_ptr }
    fn wr_ptr() -> *T unsafe { (**self).wr_ptr }
    fn rd_aux() -> *A unsafe { (**self).rd_aux }
    fn next_dirty() -> handle<T,A> unsafe { (**self).next_dirty }

    fn set_rd_ptr(t: *T) unsafe { (**self).rd_ptr = t; }
    fn set_wr_ptr(t: *T) unsafe { (**self).wr_ptr = t; }
    fn set_rd_aux(t: *A) unsafe { (**self).rd_aux = t; }
    fn set_next_dirty(+h: handle<T,A>) unsafe { (**self).next_dirty = h; }

    fn is_null() -> bool { (*self).is_null() }
    fn is_not_null() -> bool { (*self).is_not_null() }
}

impl reader_methods<T:send,A> for handle<T,A> {
    #[doc(str = "access the reader's view of the handle's data")]
    fn rd<U>(f: fn(T) -> U) -> U unsafe {
        f(*self.rd_ptr())
    }

    #[doc(str = "true if auxiliary data is associated with this handle")]
    fn has_aux() -> bool unsafe {
        self.rd_aux().is_not_null()
    }

    #[doc(str = "set the auxiliary data associated with this handle.

    **Warning:** the reader is responsible for keeping this data live!
    ")]
    fn set_aux(p: @A) unsafe {
        let p2 = p;
        unsafe::forget(p2); // Bump the reference count.

        (**self).rd_aux = ptr::addr_of(*p);
    }

    #[doc(str = "access the auxiliary data associated with this handle.")]
    fn aux<U>(f: fn(A) -> U) -> U unsafe {
        assert self.has_aux();
        f(*self.rd_aux())
    }
}

impl private_methods<T: copy send,A> for scope<T,A> {
    fn clone(v: *T) -> *T unsafe {
        let n: *mut T =
            unsafe::reinterpret_cast(
                libc::calloc(sys::size_of::<T>(), 1u));

        // n.b.: this assignment will run the drop glue for <T,A>.
        // *Hopefully* the fact that everything is initialized to NULL
        // by calloc will make this ok.  We may have to make the take
        // glue be tolerant.
        *n = unsafe{*v};

        ret unsafe::reinterpret_cast(n);
    }
}

unsafe fn free<T:send>(t: *T) {
    let _x <- *unsafe::reinterpret_cast::<*T,*mut T>(t);
    libc::free(unsafe::reinterpret_cast(t));
}

unsafe fn free_handle<T:send,A>(h: handle<T,A>) {
    free(h.rd_ptr());
    if h.wr_ptr() != h.rd_ptr() { free(h.wr_ptr()); }
}

fn null_handle<T:send,A>() -> handle<T,A> {
    _handle(ptr::null())
}

fn scope<T:send,A>() -> scope<T,A> {
    @scope_rsrc({mut layout_active: false,
                 mut free_list: [],
                 mut first_dirty: null_handle()})
}

impl writer_methods<T:copy send,A> for scope<T,A> {
    fn is_reader_forked() -> bool {
        self.layout_active
    }

    fn reader_forked() {
        assert !self.layout_active;
        assert self.first_dirty.is_null();
        self.layout_active = true;
    }

    fn reader_joined() unsafe {
        assert self.layout_active;

        if self.first_dirty.is_not_null() {
            let mut handle = self.first_dirty;
            while (*handle).is_not_null() {
                free(handle.rd_ptr());

                handle.set_rd_ptr(handle.wr_ptr());
                let next_handle = handle.next_dirty();
                handle.set_next_dirty(null_handle());
                handle = next_handle;
            }
            self.first_dirty = null_handle();
        }

        assert self.first_dirty.is_null();
        self.layout_active = false;
    }

    fn rd<U>(h: handle<T,A>, f: fn(T) -> U) -> U unsafe {
        // Use the wr_ptr, which may be more up to date than the
        // rd_ptr or may not
        f(*h.wr_ptr())
    }

    fn wr<U>(h: handle<T,A>, f: fn(T) -> U) -> U unsafe {
        if self.layout_active && h.rd_ptr() == h.wr_ptr() {
            #debug["marking handle %? as dirty", h];
            h.set_wr_ptr(self.clone(h.rd_ptr()));
            h.set_next_dirty(self.first_dirty);
            self.first_dirty = h;
        }
        f(*h.wr_ptr())
    }

    fn handle(v: T) -> handle<T,A> unsafe {
        let d: *handle_data<T,A> =
            unsafe::reinterpret_cast(
                libc::malloc(sys::size_of::<handle_data<T,A>>()));
        (*d).rd_ptr = self.clone(ptr::addr_of(v));
        (*d).wr_ptr = (*d).rd_ptr;
        (*d).rd_aux = ptr::null();
        (*d).next_dirty = null_handle();
        let h = _handle(d);
        self.free_list += [h];
        ret h;
    }
}

#[cfg(test)]
mod test {

    type animal = {name: str, species: species};
    enum species {
        chicken(~chicken),
        bull(~bull)
    }
    type chicken = {mut eggs_per_day:uint};
    type bull = {mut horns:uint};

    type processed = {flag: bool};

    type animal_scope = scope<animal, processed>;

    #[test]
    fn handles_get_freed() {
        let s: animal_scope = scope();
        s.handle({name:"henrietta", species:chicken(~{mut eggs_per_day:22u})});
        s.handle({name:"ferdinand", species:bull(~{mut horns:3u})});
    }

    fn mutate(a: animal) {
        alt a.species {
          chicken(c) { c.eggs_per_day += 1u; }
          bull(c) { c.horns += 1u; }
        }
    }

    fn read_characteristic(a: animal) -> uint {
        alt a.species {
          chicken(c) { c.eggs_per_day }
          bull(c) { c.horns }
        }
    }

    #[test]
    fn interspersed_execution() {
        let s: animal_scope = scope();
        let henrietta =
            s.handle({name:"henrietta",
                      species:chicken(~{mut eggs_per_day:0u})});
        let ferdinand =
            s.handle({name:"ferdinand",
                      species:bull(~{mut horns:0u})});

        let iter1 = 3u;
        let iter2 = 22u;
        let read_port = comm::port();
        let read_chan = comm::chan(read_port);

        // fire up a reader task
        for uint::range(0u, iter1) { |i|
            s.reader_forked();
            let wait_chan = task::spawn_listener {|wait_port|
                for uint::range(0u, iter2) { |_i|
                    comm::send(read_chan, henrietta.rd(read_characteristic));
                    comm::send(read_chan, ferdinand.rd(read_characteristic));
                    comm::recv(wait_port);
                }
            };

            let hrc = henrietta.rd(read_characteristic);
            assert hrc == (i * iter2);

            let frc = ferdinand.rd(read_characteristic);
            assert frc == i * iter2;

            for uint::range(0u, iter2) { |_i|
                assert hrc == comm::recv(read_port);
                s.wr(henrietta, mutate);
                assert frc == comm::recv(read_port);
                s.wr(ferdinand, mutate);
                comm::send(wait_chan, ());
            }
            s.reader_joined();
        }

        assert henrietta.rd(read_characteristic) == iter1 * iter2;
        assert ferdinand.rd(read_characteristic) == iter1 * iter2;
    }

}
