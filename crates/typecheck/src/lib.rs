use std::collections::HashMap;

use japp_util::Spanned;
use log::trace;
use parser::{Ident, Program};

pub mod ast;

use ast::TypedProgram;

use crate::ast::Typed;

#[derive(Debug)]
pub enum TypeError {
    UnknownIdent { ident: String },
    UnknownFunction { ident: String },
    IdentAlreadyDefined { ident: String },
    FunctionAlreadyDefined { ident: String },
}

pub fn typecheck(mut program: Program) -> Result<TypedProgram, TypeError> {
    let mut typed = Vec::with_capacity(program.declarations.len());
    let mut env = TypeEnv::new();

    for (_, decl) in program.declarations.drain() {
        let typed_decl = decl.typecheck(&mut env)?;
        typed.push(typed_decl);
    }

    Ok(TypedProgram {
        declarations: typed,
    })
}

struct TypeEnv {
    functions: HashMap<String, (Vec<Type>, Type)>,
    types: HashMap<String, Type>,
}

pub type Type = usize;

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::from_iter(
                ["null", "i64", "f64", "bool"]
                    .into_iter()
                    .enumerate()
                    .map(|(a, b)| (b.to_string(), a)),
            ),
        }
    }

    pub fn get_null(&self) -> Type {
        *self.types.get("null").unwrap()
    }

    pub fn get_int(&self) -> Type {
        *self.types.get("i64").unwrap()
    }

    pub fn get_bool(&self) -> Type {
        *self.types.get("bool").unwrap()
    }

    pub fn insert_ident(&mut self, ident: &Ident, ty: Type) -> Result<(), TypeError> {
        trace!("Inserting ident with name {} and ty {ty}", ident.inner());

        if let Some(_) = self.types.insert(ident.inner().to_string(), ty) {
            Err(TypeError::IdentAlreadyDefined {
                ident: ident.inner().to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub fn insert_function(
        &mut self,
        ident: &Ident,
        args: Vec<Type>,
        ret_ty: Type,
    ) -> Result<(), TypeError> {
        trace!("Inserting function with name {}", ident.inner());

        if let Some(_) = self
            .functions
            .insert(ident.inner().to_string(), (args, ret_ty))
        {
            Err(TypeError::FunctionAlreadyDefined {
                ident: ident.inner().to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub fn lookup_ident(&self, ident: &Ident) -> Result<Type, TypeError> {
        trace!("Finding ident with name {}", ident.inner());

        self.types
            .get(ident.inner())
            .copied()
            .ok_or(TypeError::UnknownIdent {
                ident: ident.inner().to_string(),
            })
    }

    pub fn lookup_fn(&self, ident: &Ident) -> Result<&(Vec<Type>, Type), TypeError> {
        trace!("Finding function with name {}", ident.inner());

        self.functions
            .get(ident.inner())
            .ok_or(TypeError::UnknownFunction {
                ident: ident.inner().to_string(),
            })
    }
}

trait TypeCheck {
    type Out;

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError>;
}

impl<'a> TypeCheck for parser::Decl<'a> {
    type Out = ast::Decl<'a>;

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError> {
        todo!()
        // match self {
        //     parser::Decl::Const { ident, rhs } => {
        //         let rhs = rhs.typecheck(env)?;
        //
        //         env.insert_ident(&ident, rhs.ty)?;
        //         Ok(ast::Decl::Const { ident, rhs })
        //     }
        //     parser::Decl::Fn {
        //         ident,
        //         type_def,
        //         rows,
        //     } => {
        //         let (args, return_ty) = match type_def {
        //             Some(x) => x.inner.typecheck(env)?,
        //             None => todo!("Function does not have a type def"),
        //         };
        //         // TODO: Need to insert arg types in env
        //
        //         env.insert_function(&ident, args.clone(), return_ty)?;
        //
        //         let mut typed_rows = Vec::with_capacity(rows.len());
        //         for row in rows {
        //             let ty_row = row.typecheck(env)?;
        //
        //             typed_rows.push(ty_row);
        //         }
        //
        //         Ok(ast::Decl::Fn {
        //             ident,
        //             args,
        //             return_ty,
        //             rows: typed_rows,
        //         })
        //     }
        // }
    }
}

impl TypeCheck for parser::Type<'_> {
    type Out = (Vec<Type>, Type);

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError> {
        match self {
            parser::Type::Ident(ident) => {
                let id = env.lookup_ident(&ident)?;

                Ok((Vec::new(), id))
            }
            parser::Type::Fn { mut args } => {
                let return_ty = args.pop().ok_or_else(|| todo!())?;

                let mut args_ty = Vec::with_capacity(args.len());
                for arg in args {
                    let arg_ty = arg.inner.typecheck(env)?;
                    args_ty.push(arg_ty.1);
                }

                Ok((args_ty, return_ty.inner.typecheck(env)?.1))
            }
            parser::Type::Paren { inner } => todo!(),
            parser::Type::Refined { ident, args } => todo!(),
        }
    }
}

impl<'a> TypeCheck for parser::Expr<'a> {
    type Out = ast::Typed<ast::Expr<'a>>;

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError> {
        match self {
            parser::Expr::Binary { lhs, op, rhs } => {
                let (ty_args, ret) = env.lookup_fn(&op)?.clone();

                let args = vec![lhs, rhs];

                if ty_args.len() != args.len() {
                    todo!("Incorrect number of arguments")
                }

                let mut new_args = Vec::with_capacity(args.len());
                for (arg, ty_arg) in args.into_iter().zip(ty_args) {
                    let arg = arg.typecheck(env)?;

                    if arg.ty != ty_arg {
                        todo!("Incorrect type for argument");
                    }

                    new_args.push(arg);
                }

                Ok(Typed::new(
                    ret,
                    ast::Expr::FCall {
                        ident: op,
                        args: new_args,
                    },
                ))
            }
            parser::Expr::Match { var, body } => {
                todo!()
            }
            parser::Expr::Prefix { op, rhs } => {
                let (ty_args, ret) = env.lookup_fn(&op)?.clone();

                let args = vec![rhs];

                if ty_args.len() != args.len() {
                    todo!("Incorrect number of arguments")
                }

                let mut new_args = Vec::with_capacity(args.len());
                for (arg, ty_arg) in args.into_iter().zip(ty_args) {
                    let arg = arg.typecheck(env)?;

                    if arg.ty != ty_arg {
                        todo!("Incorrect type for argument");
                    }

                    new_args.push(arg);
                }

                Ok(Typed::new(
                    ret,
                    ast::Expr::FCall {
                        ident: op,
                        args: new_args,
                    },
                ))
            }
            parser::Expr::FCall { ident, args } => {
                let (ty_args, ret) = env.lookup_fn(&ident)?.clone();

                if ty_args.len() != args.len() {
                    todo!("Incorrect number of arguments")
                }

                let mut new_args = Vec::with_capacity(args.len());
                for (arg, ty_arg) in args.into_iter().zip(ty_args) {
                    let arg = arg.typecheck(env)?;

                    if arg.ty != ty_arg {
                        todo!("Incorrect type for argument");
                    }

                    new_args.push(arg);
                }

                Ok(Typed::new(
                    ret,
                    ast::Expr::FCall {
                        ident,
                        args: new_args,
                    },
                ))
            }
            parser::Expr::Block { exprs, last } => {
                let exprs = exprs
                    .into_iter()
                    .map(|e| e.typecheck(env))
                    .collect::<Result<Vec<_>, _>>()?;

                let (ty, last) = match last {
                    Some(e) => {
                        let Typed { ty, inner } = e.typecheck(env)?;
                        let e = Typed::new(ty, Box::new(inner));
                        (ty, Some(e))
                    }
                    None => (env.get_null(), None),
                };

                Ok(Typed::new(ty, ast::Expr::Block { exprs, last }))
            }
            parser::Expr::Lit(spanned) => {
                let Typed { ty, inner } = spanned.inner.typecheck(env)?;
                Ok(Typed::new(
                    ty,
                    ast::Expr::Lit(Spanned::new(inner, spanned.span)),
                ))
            }
        }
    }
}

impl<'a, T: TypeCheck> TypeCheck for Box<T> {
    type Out = T::Out;

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError> {
        Ok((*self).typecheck(env)?)
    }
}

impl<'a> TypeCheck for parser::Lit<'a> {
    type Out = ast::Typed<ast::Lit<'a>>;

    fn typecheck(self, env: &mut TypeEnv) -> Result<Self::Out, TypeError> {
        match self {
            parser::Lit::Null => Ok(Typed::new(env.get_null(), ast::Lit::Null)),
            parser::Lit::Bool(x) => Ok(Typed::new(env.get_bool(), ast::Lit::Bool(x))),
            parser::Lit::Int(x) => Ok(Typed::new(env.get_int(), ast::Lit::Int(x))),
            parser::Lit::Ident(x) => Ok(Typed::new(env.lookup_ident(&x)?, ast::Lit::Ident(x))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
