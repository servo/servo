/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syntax::ptr::P;
use syntax::ast::{TyPath, Path, AngleBracketedParameters, PathSegment, Ty};

/// Matches a type with a provided string, and returns its type parameters if successful
pub fn match_ty_unwrap<'a>(ty: &'a Ty, segments: &[&str]) -> Option<&'a [P<Ty>]> {
    match ty.node {
        TyPath(Path {segments: ref seg, ..}, _, _) => {
            // So ast::Path isn't the full path, just the tokens that were provided.
            // I could muck around with the maps and find the full path
            // however the more efficient way is to simply reverse the iterators and zip them
            // which will compare them in reverse until one of them runs out of segments
            if seg.iter().rev().zip(segments.iter().rev()).all(|(a,b)| a.identifier.as_str() == *b) {
                match seg.as_slice().last() {
                    Some(&PathSegment {parameters: AngleBracketedParameters(ref a), ..}) => {
                        Some(a.types.as_slice())
                    }
                    _ => None
                }
            } else {
                None
            }
        },
        _ => None
    }
}
