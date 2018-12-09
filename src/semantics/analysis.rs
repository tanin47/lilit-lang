use syntax;
use semantics::scope;
use semantics::tree;
use std::cell::Cell;
use std::collections::HashMap;

pub fn analyse(
    m: &syntax::tree::Mod,
) -> tree::Mod {
    let module = build_mod(m);

    {
        let mut scope = scope::Scope { levels: Vec::new() };
        link_mod(&module, &mut scope);
    }
    module
}

fn link_mod(
    m: &tree::Mod,
    scope: &mut scope::Scope
) {
    scope.enter();
    for unit in &m.units {
        match unit {
            tree::ModUnit::Func { ref func } => {
                scope.declare(
                    &func.name,
                    scope::ScopeValue::Func(func.as_ref() as *const tree::Func)
                )
            },
            _ => (),
        }
    }
    for unit in &m.units {
        link_mod_unit(unit, scope)
    }
    scope.leave();
}

fn link_mod_unit(
    unit: &tree::ModUnit,
    scope: &mut scope::Scope,
) {
    match unit {
        tree::ModUnit::Func { ref func } => link_func(func, scope),
        _ => (),
    }
}

fn link_func(
    func: &tree::Func,
    scope: &mut scope::Scope,
) {
    scope.enter();

    for expr in &func.exprs {
        link_expr(expr, scope)
    }

    scope.leave();
}

fn link_expr(
    expr: &tree::Expr,
    scope: &mut scope::Scope,
) {
    match expr {
        tree::Expr::Invoke { ref invoke } => link_invoke(invoke, scope),
        tree::Expr::Assignment { ref assignment } => link_assignment(assignment, scope),
        tree::Expr::ReadVar { ref read_var } => link_readvar(read_var, scope),
        _ => (),
    }
}

fn link_readvar(
    readvar: &tree::ReadVar,
    scope: &mut scope::Scope,
) {
    let v = match scope.read_var(&readvar.name) {
        Some(v) => v,
        None => panic!("Unable to find the variable {:?}", readvar.name),
    };
    readvar.assignment_ref.set(Some(v as *const tree::Var))
}


fn link_assignment(
    assignment: &tree::Assignment,
    scope: &mut scope::Scope,
) {
    scope.declare(&assignment.var.name, scope::ScopeValue::Var(assignment.var.as_ref() as *const tree::Var));
}

fn link_invoke(
    invoke: &tree::Invoke,
    scope: &mut scope::Scope,
) {
    if invoke.name != "print" && invoke.name != "read" {
        let f = match scope.read_func(&invoke.name) {
            Some(func) => func,
            None => panic!("Unable to find the function {:?}", invoke.name),
        };
        invoke.func_ref.set(Some(f as *const tree::Func))
    }

    link_expr(&invoke.arg, scope);
}

pub fn build_mod(
    m: &syntax::tree::Mod,
) -> tree::Mod {
    let mut vec = Vec::new();

    for unit in &(*m).units {
        vec.push(build_mod_unit(unit));
    }

    tree::Mod { units: vec }

}

fn build_mod_unit(
    unit: &syntax::tree::ModUnit,
) -> tree::ModUnit {
    match unit {
        syntax::tree::ModUnit::Func(ref func) => tree::ModUnit::Func {
            func: Box::new(build_func(func))
        },
        syntax::tree::ModUnit::Class(ref class) => tree::ModUnit::Class {
            class: Box::new(build_class(class)),
        },
    }
}

fn build_class(
    class: &syntax::tree::Class,
) -> tree::Class {
    let mut method_vec = vec![];
    for method in &class.methods {
        method_vec.push(build_func(&method));
    }

    let mut extend_vec = vec![];
    for extend in &class.extends {
        extend_vec.push(extend.to_string());
    }

    tree::Class {
        name: class.name.to_string(),
        extends: extend_vec,
        methods: method_vec,
    }
}

fn build_func(
    func: &syntax::tree::Func,
) -> tree::Func {
    let mut vec = Vec::new();

    for expr in &(*func).exprs {
        vec.push(build_expr(expr))
    }

    tree::Func {
        llvm_ref: Cell::new(None),
        name: func.name.to_string(),
        exprs: vec,
    }
}

fn build_expr(
    expr: &syntax::tree::Expr,
) -> tree::Expr {
    match *expr {
        syntax::tree::Expr::Invoke(ref i) => tree::Expr::Invoke {
            invoke: Box::new(build_invoke(i)),
        },
        syntax::tree::Expr::Num(ref n) => tree::Expr::Num {
            num: Box::new(build_num(n)),
        },
        syntax::tree::Expr::Assignment(ref a) => tree::Expr::Assignment {
            assignment: Box::new(build_assignment(a))
        },
        syntax::tree::Expr::Var(ref v) => tree::Expr::ReadVar {
            read_var: Box::new(build_read_var(v))
        },
        syntax::tree::Expr::LiteralString(ref s) => tree::Expr::LiteralString {
            literal_string: Box::new(build_literal_string(s))
        }
    }
}

fn build_literal_string(
    literal_string: &syntax::tree::LiteralString,
) -> tree::LiteralString {
    tree::LiteralString {
        content: literal_string.content.to_string()
    }
}

fn build_read_var(
    var: &syntax::tree::Var,
) -> tree::ReadVar {
    tree::ReadVar {
        assignment_ref: Cell::new(None),
        name: var.name.to_string(),
    }
}

fn build_assignment(
    assignment: &syntax::tree::Assignment,
) -> tree::Assignment {
    tree::Assignment {
        var: Box::new(build_var(&assignment.var)),
        expr: Box::new(build_expr(&assignment.expr)),
    }
}

fn build_var(
    var: &syntax::tree::Var,
) -> tree::Var {
    tree::Var {
        llvm_ref: Cell::new(None),
        name: var.name.to_string(),
    }
}

fn build_invoke(
    invoke: &syntax::tree::Invoke
) -> tree::Invoke {
    tree::Invoke {
        func_ref: Cell::new(None),
        name: invoke.name.to_string(),
        arg: Box::new(build_expr(&invoke.arg)),
    }
}

fn build_num(
    num: &syntax::tree::Num
) -> tree::Num {
    tree::Num {
        value: num.value,
    }
}

//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn it_test() {
//        let var = Box::new(syntax::tree::Var { id: Box::new(syntax::tree::Id { name: "a".to_string() }) });
//        let value = Box::new(tree::Var {
//            llvm_ref: Cell::new(None),
//            id: Box::new(tree::Id { syntax: &var.id } ),
//            syntax: &var,
//        });
//
//        let read_var = Box::new(tree::ReadVar{
//            origin: Cell::new(None),
//            syntax: &var,
//        });
//
////        let mut scope = scope::Scope { levels: Vec::new() };
////        scope.enter();
//////        scope.declare(value.id.syntax.name.to_string(), scope::ScopeValue::Var(&value));
//////        read_var.origin.set(Some(scope.read_var(&read_var.syntax.id.name).unwrap()));
//////        test_declare(&value, &mut scope);
//////        test_link(&read_var, &mut scope);
////        test_wrapper(&value, &read_var, &mut scope);
////        scope.leave()
//    }
//
////    fn test_wrapper<'s, 'm:'s, 'd:'s, 'read_var_ref:'s + 'd, 'syntax:'read_var_ref>(
////        value: &'read_var_ref tree::Var<'syntax>,
////        readvar: &'d tree::ReadVar<'read_var_ref, 'syntax>,
////        scope: &'s mut scope::Scope<'read_var_ref, 'syntax>,
////    ) {
////        test_declare(value, scope);
////        test_link(readvar, scope);
////    }
////
////    fn test_declare<'s, 'm:'s, 'syntax:'m>(
////        value: &'m tree::Var<'syntax>,
////        scope: &'s mut scope::Scope<'m, 'syntax>,
////    ) {
////        scope.declare(value.id.syntax.name.to_string(), scope::ScopeValue::Var(value));
////    }
////
////    fn test_link<'s, 'm:'s, 'read_var_ref:'s, 'syntax:'read_var_ref>(
////        readvar: &'m tree::ReadVar<'read_var_ref, 'syntax>,
////        scope: &'s mut scope::Scope<'read_var_ref, 'syntax>
////    ) {
////        let v = match scope.read_var(&readvar.syntax.id.name) {
////            Some(v) => v,
////            None => panic!("Unable to find the variable {:?}", readvar.syntax.id.name),
////        };
////        readvar.origin.set(Some(&*v))
////    }
//}
