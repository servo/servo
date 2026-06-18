use std::{borrow::Cow, collections::HashMap};

use crate::ast::{Rule, VisitorMut};

/// This is the last visitor, which generates a Rust file
/// from the transformed AST.
pub struct GenerateVisitor<'a> {
    mods: HashMap<Option<Cow<'a, str>>, String>,
}

impl<'a> VisitorMut<'a> for GenerateVisitor<'a> {
    // TODO: for each rule, given it is global or prefixed. add to differet.
    fn visit_rule(&mut self, rule: &mut Rule<'a>) {
        let _ = todo!();
    }

    fn visit_ty(&mut self, ty: &mut crate::ast::Type<'a>) {}
}
