
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::ast::*;
use syntax::attr::AttrMetaMethods;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::*;

pub fn expand_heapsize(cx: &mut ExtCtxt, span: Span, mitem: &MetaItem, item: Annotatable, push: &mut FnMut(Annotatable)) {
    let trait_def = TraitDef {
        span: span,
        attributes: Vec::new(),
        path: ty::Path::new(vec!("util", "mem", "HeapSizeOf")),
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec![
            MethodDef {
                name: "heap_size_of_children",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: ty::borrowed_explicit_self(),
                args: vec!(),
                ret_ty: ty::Literal(ty::Path::new_local("usize")),
                attributes: vec!(),
                is_unsafe: false,
                combine_substructure: combine_substructure(Box::new(heapsize_substructure))
            }
        ],
        associated_types: vec![],
    };
    trait_def.expand(cx, mitem, &item, push)
}

fn heapsize_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let fields = match *substr.fields {
        Struct(ref fs) | EnumMatching(_, _, ref fs) => fs,
        _ => cx.span_bug(trait_span, "impossible substructure in `heapsize`")
    };

    fields.iter().fold(cx.expr_usize(trait_span, 0),
                       |acc, ref item| {
                        if item.attrs.iter()
                               .find(|ref a| a.check_name("ignore_heapsize"))
                               .is_some() {
                            acc
                        } else {
                            cx.expr_binary(item.span, BiAdd, acc,
                                           cx.expr_method_call(item.span,
                                                               item.self_.clone(),
                                                               substr.method_ident,
                                                               Vec::new()))
                        }
                    })
}
