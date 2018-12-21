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

fn convert_to_expr_type(return_type_name: &str, scope: &mut scope::Scope) -> tree::ExprType {
    if return_type_name == "Void" {
        tree::ExprType::Void
    } else if return_type_name == "Boolean" {
        tree::ExprType::Boolean
    } else if return_type_name == "Number" {
        tree::ExprType::Number
    } else if return_type_name == "String" {
        tree::ExprType::String
    } else {
        tree::ExprType::Class(scope.read_class(return_type_name).unwrap())
    }
}

fn link_mod(
    m: &tree::Mod,
    scope: &mut scope::Scope
) {
    scope.enter();
    for unit in &m.units {
        match unit {
            tree::ModUnit::Func { ref func } => {
                scope.declare(scope::ScopeValue::Func(func.as_ref()));
            },
            tree::ModUnit::Class { ref class } => {
                scope.declare(scope::ScopeValue::Class(class.as_ref()));
                for method in &class.methods {
                    method.parent_class_opt.set(Some(class.as_ref()));
                    scope.declare(scope::ScopeValue::Method(method));
                }
            },
            _ => (),
        }
    }
    for unit in &m.units {
        match unit {
            tree::ModUnit::Func { ref func } => {
                func.return_type.set(Some(convert_to_expr_type(&func.return_type_name, scope)));
            },
            tree::ModUnit::Class { ref class } => {
                for method in &class.methods {
                    method.return_type.set(Some(convert_to_expr_type(&method.return_type_name, scope)));
                }
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
        tree::ModUnit::Class { ref class } => link_class(class, scope),
    }
}

fn link_class(
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    scope.enter();
    for param in &class.params {
        param.tpe.set(Some(convert_to_expr_type(&param.tpe_name, scope)));
//        scope.declare(scope::ScopeValue::Var(param.var.as_ref()));
    }
    for method in &class.methods {
        link_method(method, scope);
    }
    scope.leave();
}

fn link_method(
    func: &tree::Func,
    scope: &mut scope::Scope,
) {
    scope.enter();

    for expr in &func.exprs {
        link_expr(expr, scope)
    }

    scope.leave();
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
        tree::Expr::Invoke(ref invoke) => link_invoke(invoke, scope),
        tree::Expr::LlvmInvoke(ref invoke) => link_llvm_invoke(invoke, scope),
        tree::Expr::Assignment(ref assignment) => link_assignment(assignment, scope),
        tree::Expr::ReadVar(ref read_var) => link_readvar(read_var, scope),
        tree::Expr::Comparison(ref comparison) => link_comparison(comparison, scope),
        tree::Expr::IfElse(ref if_else) => link_if_else(if_else, scope),
        tree::Expr::ClassInstance(ref class_instance) => link_class_instance(class_instance, scope),
        tree::Expr::LlvmClassInstance(ref class_instance) => link_llvm_class_instance(class_instance, scope),
        tree::Expr::DotInvoke(ref dot_invoke) => link_dot_invoke(dot_invoke, scope),
        tree::Expr::DotMember(ref dot_member) => link_dot_member(dot_member, scope),
        tree::Expr::Num(ref num) => (),
        tree::Expr::LiteralString(ref literal_string) => (),
        tree::Expr::Boolean(ref boolean) => (),
    }
}

fn link_dot_member(
    dot_member: &tree::DotMember,
    scope: &mut scope::Scope,
) {
    link_expr(&dot_member.expr, scope);

    match dot_member.expr.get_type() {
        tree::ExprType::Class(ref class) => {
            link_member(&dot_member.member, unsafe { &**class }, scope);
        },
        _ => panic!("Expecting a class for DotMember.expr"),
    }

    dot_member.tpe.set(dot_member.member.tpe.get());
}

fn link_member(
    member: &tree::Member,
    class: &tree::Class,
    scope: &mut scope::Scope
) {
    for (index, param) in class.params.iter().enumerate() {
        if param.var.name == member.name {
            member.param_index.set(Some(index as i32));
            member.tpe.set(param.tpe.get());
        }
    }
}

fn link_dot_invoke(
    dot_invoke: &tree::DotInvoke,
    scope: &mut scope::Scope,
) {
    link_expr(&dot_invoke.expr, scope);

    match dot_invoke.expr.get_type() {
        tree::ExprType::Class(ref class) => {
            link_method_invoke(&dot_invoke.invoke, unsafe { &**class }, scope);
        },
        _ => panic!("Expecting a class for DotInvoke.expr"),
    }
}

fn link_llvm_invoke(
    invoke: &tree::LlvmInvoke,
    scope: &mut scope::Scope,
) {
    for arg in &invoke.args {
        scope.enter();
        link_expr(arg, scope);
        scope.leave();
    }

    invoke.tpe.set(Some(convert_to_expr_type(&invoke.return_type, scope)));
}

fn link_llvm_class_instance(
    class_instance: &tree::LlvmClassInstance,
    scope: &mut scope::Scope,
) {
    scope.enter();
    link_expr(&class_instance.expr, scope);
    scope.leave();
}

fn link_class_instance(
    class_instance: &tree::ClassInstance,
    scope: &mut scope::Scope,
) {
    for param in &class_instance.params {
        scope.enter();
        link_expr(&param, scope);
        scope.leave();
    }
    let class = scope.read_class(&class_instance.name).unwrap();
    class_instance.tpe.set(Some(tree::ExprType::Class(class)));
    class_instance.class_ref.set(Some(class));
}

fn link_comparison(
    comparison: &tree::Comparison,
    scope: &mut scope::Scope,
) {
   link_readvar(&comparison.left, scope)
}

fn link_if_else(
    if_else: &tree::IfElse,
    scope: &mut scope::Scope,
) {
    link_comparison(&if_else.cond, scope);
    scope.enter();
    link_expr(&if_else.true_br, scope);
    scope.leave();
    scope.enter();
    link_expr(&if_else.false_br, scope);
    scope.leave();
    if_else.tpe.set(Some(if_else.true_br.get_type()))
}

fn link_readvar(
    readvar: &tree::ReadVar,
    scope: &mut scope::Scope,
) {
    let v = match scope.read_var(&readvar.name) {
        Some(v) => v,
        None => panic!("Unable to find the variable {:?}", readvar.name),
    };
    readvar.assignment_ref.set(Some(v as *const tree::Var));
    readvar.tpe.set(v.tpe.get());
}


fn link_assignment(
    assignment: &tree::Assignment,
    scope: &mut scope::Scope,
) {
    scope.declare(scope::ScopeValue::Var(assignment.var.as_ref() as *const tree::Var));
    link_expr(&assignment.expr, scope);

    assignment.var.tpe.set(Some(assignment.expr.get_type()))
}

fn link_method_invoke(
    invoke: &tree::Invoke,
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    {
        let f = match scope.read_method(&class.name, &invoke.name) {
            Some(func) => func,
            None => panic!("Unable to find the method {}.{}", class.name, invoke.name),
        };
        invoke.func_ref.set(Some(f as *const tree::Func));
        invoke.tpe.set(f.return_type.get());
    }

    for arg in &invoke.args {
        scope.enter();
        link_expr(arg, scope);
        scope.leave();
    }
}

fn link_invoke(
    invoke: &tree::Invoke,
    scope: &mut scope::Scope,
) {
    {
        let f = match scope.read_func(&invoke.name) {
            Some(func) => func,
            None => panic!("Unable to find the function {:?}", invoke.name),
        };
        invoke.func_ref.set(Some(f as *const tree::Func));
        invoke.tpe.set(f.return_type.get());
    }

    for arg in &invoke.args {
        scope.enter();
        link_expr(arg, scope);
        scope.leave();
    }
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
    let mut param_vec = vec![];
    for param in &class.params {
        param_vec.push(Box::new(build_class_param(&param)));
    }

    let mut extend_vec = vec![];
    for extend in &class.extends {
        extend_vec.push(extend.to_string());
    }

    let mut method_vec = vec![];
    for method in &class.methods {
        method_vec.push(build_func(&method));
    }

    tree::Class {
        name: class.name.to_string(),
        params: param_vec,
        extends: extend_vec,
        methods: method_vec,
        llvm_struct_type_ref: Cell::new(None),
    }
}

fn build_class_param(
    class_param: &syntax::tree::ClassParam
) -> tree::ClassParam {
    tree::ClassParam {
        var: Box::new(build_var(&class_param.var)),
        tpe_name: class_param.tpe.to_string(),
        tpe: Cell::new(None),
    }
}

fn build_method(
    func: &syntax::tree::Func,
) -> tree::Func {
    let mut args = vec![];
    for arg in &func.args {
        args.push(build_var(arg))
    }

    let mut exprs = vec![];
    for expr in &func.exprs {
        exprs.push(build_expr(expr))
    }

    tree::Func {
        llvm_ref: Cell::new(None),
        parent_class_opt: Cell::new(None),
        name: func.name.to_string(),
        args,
        return_type_name: func.return_type.to_string(),
        return_type: Cell::new(None),
        exprs,
    }
}

fn build_func(
    func: &syntax::tree::Func,
) -> tree::Func {
    let mut args = vec![];
    for arg in &func.args {
       args.push(build_var(arg))
    }

    let mut exprs = vec![];
    for expr in &func.exprs {
        exprs.push(build_expr(expr))
    }

    tree::Func {
        llvm_ref: Cell::new(None),
        parent_class_opt: Cell::new(None),
        name: func.name.to_string(),
        args,
        return_type_name: func.return_type.to_string(),
        return_type: Cell::new(None),
        exprs,
    }
}

fn build_expr(
    expr: &syntax::tree::Expr,
) -> tree::Expr {
    match *expr {
        syntax::tree::Expr::Invoke(ref i) => tree::Expr::Invoke(Box::new(build_invoke(i))),
        syntax::tree::Expr::LlvmInvoke(ref i) => tree::Expr::LlvmInvoke(Box::new(build_llvm_invoke(i))),
        syntax::tree::Expr::Num(ref n) => tree::Expr::Num(Box::new(build_num(n))),
        syntax::tree::Expr::Assignment(ref a) => tree::Expr::Assignment(Box::new(build_assignment(a))),
        syntax::tree::Expr::Var(ref v) => tree::Expr::ReadVar(Box::new(build_read_var(v))),
        syntax::tree::Expr::LiteralString(ref s) => tree::Expr::LiteralString(Box::new(build_literal_string(s))),
        syntax::tree::Expr::Boolean(ref b) => tree::Expr::Boolean(Box::new(build_boolean(b))),
        syntax::tree::Expr::Comparison(ref c) => tree::Expr::Comparison(Box::new(build_comparison(c))),
        syntax::tree::Expr::IfElse(ref if_else) => tree::Expr::IfElse(Box::new(build_if_else(if_else))),
        syntax::tree::Expr::ClassInstance(ref class_instance) => tree::Expr::ClassInstance(Box::new(build_class_instance(class_instance))),
        syntax::tree::Expr::LlvmClassInstance(ref class_instance) => tree::Expr::LlvmClassInstance(Box::new(build_llvm_class_instance(class_instance))),
        syntax::tree::Expr::DotInvoke(ref dot_invoke) => tree::Expr::DotInvoke(Box::new(build_dot_invoke(dot_invoke))),
        syntax::tree::Expr::DotMember(ref dot_member) => tree::Expr::DotMember(Box::new(build_dot_member(dot_member))),
    }
}

fn build_dot_member(
    dot_member: &syntax::tree::DotMember
) -> tree::DotMember {
    tree::DotMember {
        expr: Box::new(build_expr(&dot_member.expr)),
        member: Box::new(build_member(&dot_member.member)),
        tpe: Cell::new(None),
    }
}

fn build_member(
    member: &syntax::tree::Var
) -> tree::Member {
    tree::Member {
        name: member.name.to_string(),
        param_index: Cell::new(None),
        tpe: Cell::new(None),
    }
}

fn build_dot_invoke(
    dot_invoke: &syntax::tree::DotInvoke
) -> tree::DotInvoke {
    tree::DotInvoke {
        expr: Box::new(build_expr(&dot_invoke.expr)),
        invoke: Box::new(build_invoke(&dot_invoke.invoke)),
        tpe: Cell::new(None),
    }
}

fn build_llvm_class_instance(
    class_instance: &syntax::tree::LlvmClassInstance
) -> tree::LlvmClassInstance {
    tree::LlvmClassInstance {
        name: class_instance.name.to_string(),
        expr: Box::new(build_expr(&class_instance.expr)),
        tpe: Cell::new(None),
    }
}

fn build_class_instance(
    class_instance: &syntax::tree::ClassInstance
) -> tree::ClassInstance {
    let mut param_vec = vec![];
    for param in &class_instance.params {
       param_vec.push(Box::new(build_expr(&param)));
    }

    tree::ClassInstance {
        name: class_instance.name.to_string(),
        params: param_vec,
        tpe: Cell::new(None),
        class_ref: Cell::new(None),
    }
}

fn build_boolean(
    boolean: &syntax::tree::Boolean
) -> tree::Boolean {
    tree::Boolean {
        value: boolean.value,
        tpe: tree::ExprType::Boolean,
    }
}

fn build_comparison(
    comparison: &syntax::tree::Comparison
) -> tree::Comparison {
    tree::Comparison {
        left: Box::new(build_read_var(&comparison.left)),
        right: Box::new(build_num(&comparison.right)),
        tpe: tree::ExprType::Boolean,
    }
}

fn build_if_else(
    if_else: &syntax::tree::IfElse
) -> tree::IfElse {
    tree::IfElse {
        cond: Box::new(build_comparison(&if_else.cond)),
        true_br: Box::new(build_expr(&if_else.true_br)),
        false_br: Box::new(build_expr(&if_else.false_br)),
        tpe: Cell::new(None),
    }
}

fn build_literal_string(
    literal_string: &syntax::tree::LiteralString,
) -> tree::LiteralString {
    tree::LiteralString {
        content: literal_string.content.to_string(),
        tpe: tree::ExprType::String,
    }
}

fn build_read_var(
    var: &syntax::tree::Var,
) -> tree::ReadVar {
    tree::ReadVar {
        assignment_ref: Cell::new(None),
        name: var.name.to_string(),
        tpe: Cell::new(None),
    }
}

fn build_assignment(
    assignment: &syntax::tree::Assignment,
) -> tree::Assignment {
    let expr = Box::new(build_expr(&assignment.expr));
    tree::Assignment {
        var: Box::new(build_var(&assignment.var)),
        expr,
        tpe: Cell::new(None),
    }
}

fn build_var(
    var: &syntax::tree::Var,
) -> tree::Var {
    tree::Var {
        llvm_ref: Cell::new(None),
        tpe: Cell::new(None),
        name: var.name.to_string(),
    }
}

fn build_invoke(
    invoke: &syntax::tree::Invoke
) -> tree::Invoke {
    let mut args: Vec<tree::Expr> = vec![];

    for arg in &invoke.args {
        args.push(build_expr(arg));
    }

    tree::Invoke {
        func_ref: Cell::new(None),
        name: invoke.name.to_string(),
        args,
        tpe: Cell::new(None),
    }
}

fn build_llvm_invoke(
    invoke: &syntax::tree::LlvmInvoke
) -> tree::LlvmInvoke {
    let mut args: Vec<tree::Expr> = vec![];

    for arg in &invoke.args {
        args.push(build_expr(arg));
    }

    tree::LlvmInvoke {
        name: invoke.name.to_string(),
        is_varargs: invoke.is_varargs,
        return_type: invoke.return_type.to_string(),
        args,
        tpe: Cell::new(None),
    }
}

fn build_num(
    num: &syntax::tree::Num
) -> tree::Num {
    tree::Num {
        value: num.value,
        tpe: tree::ExprType::Number,
    }
}

