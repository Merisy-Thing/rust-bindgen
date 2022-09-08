//! Intermediate representation of variables.

use std::io;
use std::str;

use crate::callbacks::MacroParsingBehavior;
use crate::clang;
use crate::parse::{ClangSubItemParser, ParseError, ParseResult};

use super::context::BindgenContext;
use super::dot::DotAttributes;

/// A `MacroDef` is our intermediate representation of a macro definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroDef {
    /// A function-like macro.
    Fn(String),
    /// A variable-like macro.
    Var(String),
}

impl MacroDef {
    /// Get the macro name.
    pub fn name(&self) -> &str {
        match self {
            Self::Fn(name) => name,
            Self::Var(name) => name,
        }
    }
}

impl DotAttributes for MacroDef {
    fn dot_attributes<W>(
        &self,
        _ctx: &BindgenContext,
        out: &mut W,
    ) -> io::Result<()>
    where
        W: io::Write,
    {
        writeln!(out, "<tr><td>macro</td><td>true</td></tr>")
    }
}

impl ClangSubItemParser for MacroDef {
    fn parse(
        cursor: clang::Cursor,
        ctx: &mut BindgenContext,
    ) -> Result<ParseResult<Self>, ParseError> {
        use clang_sys::CXCursor_MacroDefinition;

        if cursor.kind() != CXCursor_MacroDefinition {
            return Err(ParseError::Continue);
        }

        match ctx
            .options()
            .last_callback(|c| Some(c.will_parse_macro(&cursor.spelling())))
            .unwrap_or_default()
        {
            MacroParsingBehavior::Ignore => {
                return Err(ParseError::Continue);
            }
            MacroParsingBehavior::Default => (),
        }

        let clang_tokens = cursor.tokens().iter().collect::<Vec<_>>();
        let args_boundary = if cursor.is_macro_function_like() {
            clang_tokens.iter().position(|t| {
                t.kind == clang_sys::CXToken_Punctuation && t.spelling() == b")"
            })
        } else {
            None
        };
        let mut cmacro_tokens = clang_tokens
            .iter()
            .map(|t| t.spelling())
            .collect::<Vec<_>>();
        let name = str::from_utf8(cmacro_tokens.remove(0)).unwrap();

        let args = if let Some(args_boundary) = args_boundary {
            let args: Vec<_> = cmacro_tokens
                .drain(0..args_boundary)
                .skip(1)
                .take(args_boundary - 2)
                .filter(|&token| token != b",")
                .collect();
            Some(args)
        } else {
            None
        };

        if let Some(args) = args {
            if !ctx.options().parse_callbacks.is_empty() {
                let args = args
                    .iter()
                    .map(|t| str::from_utf8(t).unwrap())
                    .collect::<Vec<_>>();

                for callbacks in &ctx.options().parse_callbacks {
                    callbacks.func_macro(name, &args, &cmacro_tokens);
                }
            }

            let fn_macro =
                cmacro::FnMacro::parse(name.as_bytes(), &args, &cmacro_tokens);
            if let Ok(fn_macro) = fn_macro {
                ctx.add_fn_macro(fn_macro);
                return Ok(ParseResult::New(
                    MacroDef::Fn(name.to_owned()),
                    Some(cursor),
                ));
            }
        } else {
            let macro_def =
                cmacro::VarMacro::parse(name.as_bytes(), &cmacro_tokens);
            if let Ok(macro_def) = macro_def {
                ctx.add_var_macro(macro_def);
                return Ok(ParseResult::New(
                    MacroDef::Var(name.to_owned()),
                    Some(cursor),
                ));
            }
        }

        Err(ParseError::Continue)
    }
}
