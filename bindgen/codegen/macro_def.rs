use cmacro::{Expr, Lit, LitInt};

use crate::callbacks::IntKind;
use crate::ir::context::BindgenContext;
use crate::ir::item::{Item, ItemCanonicalName};
use crate::ir::macro_def::MacroDef;

use super::{
    attributes, helpers::ast_ty, CodeGenerator, CodegenResult,
    MacroTypeVariation,
};

fn default_macro_constant_type(ctx: &BindgenContext, value: i128) -> IntKind {
    if value < 0 ||
        ctx.options().default_macro_constant_type ==
            MacroTypeVariation::Signed
    {
        if value < i64::MIN as i128 || value > i64::MAX as i128 {
            IntKind::I128
        } else if value < i32::MIN as i128 || value > i32::MAX as i128 {
            IntKind::I64
        } else if !ctx.options().fit_macro_constants ||
            value < i16::MIN as i128 ||
            value > i16::MAX as i128
        {
            IntKind::I32
        } else if value < i8::MIN as i128 || value > i8::MAX as i128 {
            IntKind::I16
        } else {
            IntKind::I8
        }
    } else if value > u32::MAX as i128 {
        IntKind::U64
    } else if !ctx.options().fit_macro_constants || value > u16::MAX as i128 {
        IntKind::U32
    } else if value > u8::MAX as i128 {
        IntKind::U16
    } else {
        IntKind::U8
    }
}

impl CodeGenerator for MacroDef {
    type Extra = Item;
    type Return = ();

    fn codegen(
        &self,
        ctx: &BindgenContext,
        result: &mut CodegenResult<'_>,
        item: &Item,
    ) {
        debug!("<MacroDef as CodeGenerator>::codegen: item = {:?}", item);
        debug_assert!(item.is_enabled_for_codegen(ctx));

        let canonical_name = item.canonical_name(ctx);

        let mut attrs = vec![];
        if let Some(comment) = item.comment(ctx) {
            attrs.push(attributes::doc(comment));
        }

        match self {
            Self::Fn(name) => {
                if result.seen_function(&canonical_name) {
                    return;
                }
                result.saw_function(&canonical_name);

                let mut fn_macro = ctx.fn_macro(name).unwrap().clone();
                let generated_value = match fn_macro.generate(ctx) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            "Cannot generate function macro: {:?}\n{:?}",
                            err, fn_macro
                        );
                        return;
                    }
                };

                result.push(quote! {
                    #(#attrs)*
                    #generated_value
                });
            }
            Self::Var(name) => {
                if result.seen_var(&canonical_name) {
                    return;
                }
                result.saw_var(&canonical_name);

                let canonical_ident = ctx.rust_ident(&canonical_name);

                let mut var_macro = ctx.var_macro(name).unwrap().clone();
                let (generated_value, generated_type) =
                    match var_macro.generate(ctx) {
                        Ok((value, ty)) => (value, ty),
                        Err(err) => {
                            warn!(
                                "Cannot generate variable macro: {:?}\n{:?}",
                                err, var_macro
                            );
                            return;
                        }
                    };

                match var_macro.value {
                    Expr::Literal(Lit::Int(LitInt { value, .. })) => {
                        let int_kind = ctx
                            .options()
                            .last_callback(|c| {
                                if value >= i64::MIN as i128 &&
                                    value <= i64::MAX as i128
                                {
                                    let value = value as i64;
                                    c.int_macro(self.name(), value)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| {
                                default_macro_constant_type(ctx, value)
                            });

                        let ty =
                            ast_ty::int_kind_rust_type(ctx, int_kind, None);
                        let value = if int_kind.is_signed() {
                            ast_ty::int_expr(value)
                        } else {
                            ast_ty::uint_expr(value as _)
                        };

                        result.push(quote! {
                            #(#attrs)*
                            pub const #canonical_ident : #ty = #value;
                        });
                    }
                    expr => {
                        if let Expr::Literal(Lit::String(ref s)) = expr {
                            for callbacks in &ctx.options().parse_callbacks {
                                callbacks.str_macro(self.name(), s.as_bytes());
                            }
                        }

                        if let Some(ty) = generated_type {
                            result.push(quote! {
                                #(#attrs)*
                                pub const #canonical_ident : #ty = #generated_value;
                            });
                        } else {
                            warn!(
                                "Unhandled variable macro: {} = {:?}",
                                var_macro.name, expr
                            );
                        }
                    }
                }
            }
        }
    }
}
