/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

{
    macro_rules! move_ref(
        { $x:expr } => { unsafe { let y <- *ptr::to_unsafe_ptr(*$x); y } }
    )

    macro_rules! move_val(
        { $x:expr } => { unsafe { let y <- *ptr::to_unsafe_ptr(*$x); y } }
    )

    // select!
    macro_rules! select_if(
    
        {
            $index:expr,
            $count:expr
        } => {
            fail
        };
    
        {
            $index:expr,
            $count:expr,
            $port:path => [
                $(type_this $message:path$(($(x $x: ident),+))dont_type_this*
                  -> $next:ident => { $e:expr }),+
            ]
            $(, $ports:path => [
                $(type_this $messages:path$(($(x $xs: ident),+))dont_type_this*
                  -> $nexts:ident => { $es:expr }),+
            ] )*
        } => {
            if $index == $count {
                match pipes::try_recv($port) {
                  $(Some($message($($(ref $x,)+)* ref next)) => {
                    // FIXME (#2329) we really want move out of enum here.
                    let $next = move_ref!(next);
                    $e
                  })+
                  _ => fail
                }
            } else {
                select_if!(
                    $index,
                    $count + 1
                    $(, $ports => [
                        $(type_this $messages$(($(x $xs),+))dont_type_this*
                          -> $nexts => { $es }),+
                    ])*
                )
            }
        };
    )
    
    macro_rules! select(
        {
            $( $port:path => {
                $($message:path$(($($x: ident),+))dont_type_this*
                  -> $next:ident $e:expr),+
            } )+
        } => {
            let index = pipes::selecti([$(($port).header()),+]/_);
            select_if!(index, 0 $(, $port => [
                $(type_this $message$(($(x $x),+))dont_type_this* -> $next => { $e }),+
            ])+)
        }
    )
}
