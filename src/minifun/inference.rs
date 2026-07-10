use std::collections::{HashMap, HashSet};

use crate::minifun::ast::{BinOp, Term};

pub type TypeVar = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MonoType {
    Int,
    Bool,
    Var(TypeVar),
    Fun(Box<MonoType>, Box<MonoType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyType {
    pub vars: Vec<TypeVar>,
    pub ty: MonoType,
}

pub type Substitution = HashMap<TypeVar, MonoType>;
pub type TypeEnvironment = HashMap<String, PolyType>;

#[derive(Debug, Default)]
pub struct TypeVarGenerator {
    counter: usize,
}

impl TypeVarGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn fresh(&mut self) -> MonoType {
        let name = format!("a{}", self.counter);
        self.counter += 1;
        MonoType::Var(name)
    }
}

pub fn mono(ty: MonoType) -> PolyType {
    PolyType { vars: vec![], ty }
}

pub fn free_type_vars_mono(ty: &MonoType) -> HashSet<TypeVar> {
    match ty {
        MonoType::Int | MonoType::Bool => HashSet::new(),
        MonoType::Var(v) => HashSet::from([v.clone()]),
        MonoType::Fun(t1, t2) => {
            let mut vars = free_type_vars_mono(t1);
            vars.extend(free_type_vars_mono(t2));
            vars
        }
    }
}

pub fn free_type_vars_poly(poly: &PolyType) -> HashSet<TypeVar> {
    let mut vars = free_type_vars_mono(&poly.ty);

    for v in &poly.vars {
        vars.remove(v);
    }

    vars
}

pub fn free_type_vars_env(env: &TypeEnvironment) -> HashSet<TypeVar> {
    let mut vars = HashSet::new();

    for poly in env.values() {
        vars.extend(free_type_vars_poly(poly));
    }

    vars
}

pub fn apply_subst_mono(subst: &Substitution, ty: &MonoType) -> MonoType {
    match ty {
        MonoType::Int => MonoType::Int,
        MonoType::Bool => MonoType::Bool,

        MonoType::Var(v) => subst
            .get(v)
            .cloned()
            .unwrap_or_else(|| MonoType::Var(v.clone())),

        MonoType::Fun(t1, t2) => MonoType::Fun(
            Box::new(apply_subst_mono(subst, t1)),
            Box::new(apply_subst_mono(subst, t2)),
        ),
    }
}

pub fn apply_subst_poly(subst: &Substitution, poly: &PolyType) -> PolyType {
    let mut filtered = subst.clone();

    for v in &poly.vars {
        filtered.remove(v);
    }

    PolyType {
        vars: poly.vars.clone(),
        ty: apply_subst_mono(&filtered, &poly.ty),
    }
}

pub fn apply_subst_env(subst: &Substitution, env: &TypeEnvironment) -> TypeEnvironment {
    env.iter()
        .map(|(name, poly)| (name.clone(), apply_subst_poly(subst, poly)))
        .collect()
}

pub fn compose_subst(s1: &Substitution, s2: &Substitution) -> Substitution {
    let mut result = HashMap::new();

    for (var, ty) in s2 {
        result.insert(var.clone(), apply_subst_mono(s1, ty));
    }

    for (var, ty) in s1 {
        result.insert(var.clone(), ty.clone());
    }

    result
}

pub fn inst(poly: &PolyType, type_gen: &mut TypeVarGenerator) -> MonoType {
    let mut subst = HashMap::new();

    for var in &poly.vars {
        subst.insert(var.clone(), type_gen.fresh());
    }

    apply_subst_mono(&subst, &poly.ty)
}

pub fn gener(env: &TypeEnvironment, ty: &MonoType) -> PolyType {
    let env_vars = free_type_vars_env(env);
    let ty_vars = free_type_vars_mono(ty);

    let vars = ty_vars.difference(&env_vars).cloned().collect::<Vec<_>>();

    PolyType {
        vars,
        ty: ty.clone(),
    }
}

fn occurs(var: &TypeVar, ty: &MonoType) -> bool {
    free_type_vars_mono(ty).contains(var)
}

fn bind(var: &TypeVar, ty: &MonoType) -> Result<Substitution, String> {
    if ty == &MonoType::Var(var.clone()) {
        Ok(HashMap::new())
    } else if occurs(var, ty) {
        Err(format!(
            "Occurs check failed: cannot construct infinite type {} = {:?}",
            var, ty
        ))
    } else {
        let mut subst = HashMap::new();
        subst.insert(var.clone(), ty.clone());
        Ok(subst)
    }
}

pub fn unify(t1: &MonoType, t2: &MonoType) -> Result<Substitution, String> {
    match (t1, t2) {
        (MonoType::Int, MonoType::Int) => Ok(HashMap::new()),
        (MonoType::Bool, MonoType::Bool) => Ok(HashMap::new()),

        (MonoType::Var(v), ty) => bind(v, ty),
        (ty, MonoType::Var(v)) => bind(v, ty),

        (MonoType::Fun(a1, r1), MonoType::Fun(a2, r2)) => {
            let s1 = unify(a1, a2)?;

            let r1_sub = apply_subst_mono(&s1, r1);
            let r2_sub = apply_subst_mono(&s1, r2);

            let s2 = unify(&r1_sub, &r2_sub)?;

            Ok(compose_subst(&s2, &s1))
        }

        _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
    }
}

pub fn infer(
    env: &TypeEnvironment,
    term: &Term,
    type_gen: &mut TypeVarGenerator,
) -> Result<(Substitution, MonoType), String> {
    match term {
        Term::Int(_) => Ok((HashMap::new(), MonoType::Int)),

        Term::Bool(_) => Ok((HashMap::new(), MonoType::Bool)),

        Term::Var(name) => {
            let poly = env
                .get(name)
                .ok_or_else(|| format!("Unbound variable '{}'", name))?;

            Ok((HashMap::new(), inst(poly, type_gen)))
        }

        Term::BinOp(left, op, right) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                let (s1, t1) = infer(env, left, type_gen)?;
                let env1 = apply_subst_env(&s1, env);

                let (s2, t2) = infer(&env1, right, type_gen)?;

                let s3 = unify(&apply_subst_mono(&s2, &t1), &MonoType::Int)?;
                let s4 = unify(&apply_subst_mono(&s3, &t2), &MonoType::Int)?;

                let subst = compose_subst(&s4, &compose_subst(&s3, &compose_subst(&s2, &s1)));

                Ok((subst, MonoType::Int))
            }

            BinOp::And => {
                let (s1, t1) = infer(env, left, type_gen)?;
                let env1 = apply_subst_env(&s1, env);

                let (s2, t2) = infer(&env1, right, type_gen)?;

                let s3 = unify(&apply_subst_mono(&s2, &t1), &MonoType::Bool)?;
                let s4 = unify(&apply_subst_mono(&s3, &t2), &MonoType::Bool)?;

                let subst = compose_subst(&s4, &compose_subst(&s3, &compose_subst(&s2, &s1)));

                Ok((subst, MonoType::Bool))
            }

            BinOp::Less => {
                let (s1, t1) = infer(env, left, type_gen)?;
                let env1 = apply_subst_env(&s1, env);

                let (s2, t2) = infer(&env1, right, type_gen)?;

                let s3 = unify(&apply_subst_mono(&s2, &t1), &MonoType::Int)?;
                let s4 = unify(&apply_subst_mono(&s3, &t2), &MonoType::Int)?;

                let subst = compose_subst(&s4, &compose_subst(&s3, &compose_subst(&s2, &s1)));

                Ok((subst, MonoType::Bool))
            }
        },

        Term::Not(inner) => {
            let (s1, t1) = infer(env, inner, type_gen)?;
            let s2 = unify(&t1, &MonoType::Bool)?;
            let subst = compose_subst(&s2, &s1);

            Ok((subst, MonoType::Bool))
        }

        Term::If(cond, then_term, else_term) => {
            let (s1, t_cond) = infer(env, cond, type_gen)?;
            let s_bool = unify(&t_cond, &MonoType::Bool)?;

            let env1 = apply_subst_env(&compose_subst(&s_bool, &s1), env);

            let (s2, t_then) = infer(&env1, then_term, type_gen)?;
            let env2 = apply_subst_env(&s2, &env1);

            let (s3, t_else) = infer(&env2, else_term, type_gen)?;

            let t_then_sub = apply_subst_mono(&s3, &t_then);
            let s4 = unify(&t_then_sub, &t_else)?;

            let subst = compose_subst(
                &s4,
                &compose_subst(&s3, &compose_subst(&s2, &compose_subst(&s_bool, &s1))),
            );

            let final_type = apply_subst_mono(&subst, &t_else);

            Ok((subst, final_type))
        }

        Term::Fun { param, body, .. } => {
            let param_type = type_gen.fresh();

            let mut env1 = env.clone();
            env1.insert(param.clone(), mono(param_type.clone()));

            let (s1, body_type) = infer(&env1, body, type_gen)?;

            let final_param_type = apply_subst_mono(&s1, &param_type);

            Ok((
                s1,
                MonoType::Fun(Box::new(final_param_type), Box::new(body_type)),
            ))
        }

        Term::App(fun_term, arg_term) => {
            let result_type = type_gen.fresh();

            let (s1, fun_type) = infer(env, fun_term, type_gen)?;
            let env1 = apply_subst_env(&s1, env);

            let (s2, arg_type) = infer(&env1, arg_term, type_gen)?;

            let expected_fun_type =
                MonoType::Fun(Box::new(arg_type), Box::new(result_type.clone()));

            let s3 = unify(&apply_subst_mono(&s2, &fun_type), &expected_fun_type)?;

            let subst = compose_subst(&s3, &compose_subst(&s2, &s1));
            let final_type = apply_subst_mono(&subst, &result_type);

            Ok((subst, final_type))
        }

        Term::Let(name, value, body) => {
            let (s1, value_type) = infer(env, value, type_gen)?;

            let env1 = apply_subst_env(&s1, env);
            let generalized = gener(&env1, &apply_subst_mono(&s1, &value_type));

            let mut env2 = env1;
            env2.insert(name.clone(), generalized);

            let (s2, body_type) = infer(&env2, body, type_gen)?;

            let subst = compose_subst(&s2, &s1);

            Ok((subst, body_type))
        }

        Term::LetFun {
            name,
            param,
            body,
            in_term,
            ..
        } => {
            let param_type = type_gen.fresh();
            let result_type = type_gen.fresh();

            let fun_type =
                MonoType::Fun(Box::new(param_type.clone()), Box::new(result_type.clone()));

            let mut env1 = env.clone();
            env1.insert(name.clone(), mono(fun_type.clone()));
            env1.insert(param.clone(), mono(param_type.clone()));

            let (s1, inferred_body_type) = infer(&env1, body, type_gen)?;

            let s2 = unify(&apply_subst_mono(&s1, &result_type), &inferred_body_type)?;

            let subst_fun = compose_subst(&s2, &s1);

            let env2_base = apply_subst_env(&subst_fun, env);
            let final_fun_type = apply_subst_mono(&subst_fun, &fun_type);
            let generalized = gener(&env2_base, &final_fun_type);

            let mut env2 = env2_base;
            env2.insert(name.clone(), generalized);

            let (s3, in_type) = infer(&env2, in_term, type_gen)?;

            let subst = compose_subst(&s3, &subst_fun);

            Ok((subst, in_type))
        }
    }
}
pub fn typecheck(term: &Term) -> Result<MonoType, String> {
    let env = TypeEnvironment::new();
    let mut type_gen = TypeVarGenerator::new();

    let (subst, ty) = infer(&env, term, &mut type_gen)?;

    Ok(apply_subst_mono(&subst, &ty))
}
