use syntax;
use semantics::scope;
use semantics::tree;
use std::cell::Cell;
use std::collections::HashMap;

pub struct Context {
    pub in_llvm_mode: bool,
}

pub fn analyse(
    m: &syntax::tree::Mod,
) -> tree::Mod {
    let module = {
        let context = Context {
            in_llvm_mode: false
        };
        build_mod(m, &context)
    };

    {
        let mut scope = scope::Scope { levels: Vec::new() };
        link_mod(&module, &mut scope);
    }
    module
}

fn convert_to_expr_type(return_type_name: &str, scope: &mut scope::Scope) -> tree::ExprType {
    if return_type_name == "Void" {
        tree::ExprType::Void
    } else if return_type_name == "LlvmBoolean" {
        tree::ExprType::LlvmBoolean
    } else if return_type_name == "LlvmNumber" {
        tree::ExprType::LlvmNumber
    } else if return_type_name == "LlvmString" {
        tree::ExprType::LlvmString
    } else if return_type_name == "LlvmArray" {
        tree::ExprType::LlvmArray
    } else if return_type_name == "LlvmChar" {
        tree::ExprType::LlvmChar
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
            tree::ModUnit::Func(ref func) => {
                scope.declare(scope::ScopeValue::Func(func.as_ref()));
            },
            tree::ModUnit::Class(ref class) => {

                match class.name.as_ref() {
                    "Number" => m.number_class.set(Some(class.as_ref())),
                    "@I32" => m.llvm_number_class.set(Some(class.as_ref())),
                    "Boolean" => m.boolean_class.set(Some(class.as_ref())),
                    "@Boolean" => m.llvm_boolean_class.set(Some(class.as_ref())),
                    "String" => m.string_class.set(Some(class.as_ref())),
                    "@String" => m.llvm_string_class.set(Some(class.as_ref())),
                    "Array" => m.array_class.set(Some(class.as_ref())),
                    "@Array" => m.llvm_array_class.set(Some(class.as_ref())),
                    "Char" => m.char_class.set(Some(class.as_ref())),
                    "@Char" => m.llvm_char_class.set(Some(class.as_ref())),
                    _ => (),
                }

                scope.declare(scope::ScopeValue::Class(class.as_ref()));
                for method in &class.methods {
                    method.parent_class_opt.set(Some(class.as_ref()));

                    if method.is_static {
                        scope.declare(scope::ScopeValue::StaticMethod(method));
                    } else {
                        scope.declare(scope::ScopeValue::Method(method));
                    }
                }
            },
        }
    }
    for unit in &m.units {
        match unit {
            tree::ModUnit::Func(ref func) => {
                func.return_type.set(Some(convert_to_expr_type(&func.return_type_name, scope)));
            },
            tree::ModUnit::Class(ref class) => {
                for method in &class.methods {
                    method.return_type.set(Some(convert_to_expr_type(&method.return_type_name, scope)));
                }
            },
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
        tree::ModUnit::Func(ref func) => link_func(func, scope),
        tree::ModUnit::Class(ref class) => link_class(class, scope),
    }
}

fn link_class(
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    scope.enter();
    for param in &class.params {
        param.tpe.set(Some(convert_to_expr_type(&param.tpe_name, scope)));
    }
    for method in &class.methods {
        if method.is_static {
            link_static_method(method, class, scope);
        } else {
            link_method(method, class, scope);
        }
    }
    scope.leave();
}

fn link_static_method(
    func: &tree::Func,
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    scope.enter();
    for param in &func.params {
        param.var.tpe.set(Some(convert_to_expr_type(&param.tpe_name, scope)));
        param.tpe.set(param.var.tpe.get());
        scope.declare(scope::ScopeValue::Var(param.var.as_ref()));
    }

    for expr in &func.exprs {
        link_expr(expr, scope)
    }

    scope.leave();
}

fn link_method(
    func: &tree::Func,
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    scope.enter();
    let this_param = &func.params[0];

    for param in &class.params {
        scope.declare(scope::ScopeValue::Member(param.var.as_ref(), this_param.var.as_ref()));
    }

    for param in &func.params {
        param.var.tpe.set(Some(convert_to_expr_type(&param.tpe_name, scope)));
        param.tpe.set(param.var.tpe.get());
        scope.declare(scope::ScopeValue::Var(param.var.as_ref()));
    }

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

    for param in &func.params {
        scope.declare(scope::ScopeValue::Var(param.var.as_ref()));
    }

    for param in &func.params {
        param.var.tpe.set(Some(convert_to_expr_type(&param.tpe_name, scope)));
        param.tpe.set(param.var.tpe.get());
        scope.declare(scope::ScopeValue::Var(param.var.as_ref()));
    }

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
        tree::Expr::Reassignment(ref reassignment) => link_reassignment(reassignment, scope),
        tree::Expr::ReadVar(ref read_var) => link_readvar(read_var, scope),
        tree::Expr::IfElse(ref if_else) => link_if_else(if_else, scope),
        tree::Expr::ClassInstance(ref class_instance) => link_class_instance(class_instance, scope),
        tree::Expr::LlvmClassInstance(ref class_instance) => link_llvm_class_instance(class_instance, scope),
        tree::Expr::DotInvoke(ref dot_invoke) => link_dot_invoke(dot_invoke, scope),
        tree::Expr::DotMember(ref dot_member) => link_dot_member(dot_member, scope),
        tree::Expr::LlvmString(ref literal_string) => (),
        tree::Expr::LlvmBoolean(ref boolean) => (),
        tree::Expr::LlvmNumber(ref value) => (),
        tree::Expr::LlvmChar(ref c) => (),
        tree::Expr::LlvmArray(ref array) => link_llvm_array(array, scope),
        tree::Expr::While(ref whle) => link_while(whle, scope),
        tree::Expr::StaticClassInstance(ref s) => link_static_class_instance(s, scope),
    }
}

fn link_while(
    whle: &tree::While,
    scope: &mut scope::Scope
) {
    link_expr(&whle.cond, scope);

    for expr in &whle.exprs {
        link_expr(expr, scope);
    }
}

fn link_llvm_array(
    array: &tree::LlvmArray,
    scope: &mut scope::Scope,
) {
    for item in &array.items {
        link_expr(item, scope);
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
        tree::ExprType::StaticClass(ref class) => {
            link_static_method_invoke(&dot_invoke.invoke, unsafe { &**class }, scope);
        },
        _ => panic!("Expecting a class for DotInvoke.expr"),
    }

    dot_invoke.tpe.set(dot_invoke.invoke.tpe.get());
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

    let class = scope.read_class(&invoke.return_type_name).unwrap();
    invoke.return_type.set(Some(tree::ExprType::Class(class)));
}

fn link_static_class_instance(
    instance: &tree::StaticClassInstance,
    scope: &mut scope::Scope,
) {
    let class = scope.read_class(&instance.name).unwrap();
    instance.class_ref.set(Some(class));
    instance.tpe.set(Some(tree::ExprType::StaticClass(class)));
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

fn link_llvm_class_instance(
    class_instance: &tree::LlvmClassInstance,
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

fn link_if_else(
    if_else: &tree::IfElse,
    scope: &mut scope::Scope,
) {
    link_expr(&if_else.cond, scope);

    match if_else.cond.get_type() {
        tree::ExprType::Class(class_ptr) => {
            let class = unsafe { &*class_ptr };
            if class.name != "Boolean" {
                panic!("Expect Boolean, found {:?}", class.name)
            }
        },
        x => panic!("Expect semantics::tree::ExprType::Boolean, found {:?}", x),
    };

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
    match scope.read(&readvar.name) {
        Some(scope::ScopeValue::Var(v_ptr)) => {
            let v = unsafe { &**v_ptr };
            readvar.assignment_ref.set(Some(v));
            readvar.tpe.set(v.tpe.get());
        },
        Some(scope::ScopeValue::Member(m_ptr, this_ptr)) => {
            let this = unsafe { &**this_ptr } ;
            let m = unsafe { &**m_ptr } ;
            readvar.assignment_ref.set(Some(this));
            match this.tpe.get().unwrap() {
                tree::ExprType::Class(ref class_ptr) => {
                    let class = unsafe { &**class_ptr };
                    for (index, param) in class.params.iter().enumerate() {
                        if param.var.name == m.name {
                            readvar.member_param_index.set(Some(index as i32));
                            readvar.tpe.set(param.tpe.get());
                        }
                    }
                },
                _ => panic!("Expecting a class for DotMember.expr"),
            }
        },
        _ => panic!("Unable to find the variable {:?}", readvar.name),
    };
}

fn link_reassignment(
    reassignment: &tree::Reassignment,
    scope: &mut scope::Scope,
) {
    link_readvar(&reassignment.var, scope);
    link_expr(&reassignment.expr, scope);

    reassignment.var.tpe.set(Some(reassignment.expr.get_type()))
}

fn link_assignment(
    assignment: &tree::Assignment,
    scope: &mut scope::Scope,
) {
    scope.declare(scope::ScopeValue::Var(assignment.var.as_ref() as *const tree::Var));
    link_expr(&assignment.expr, scope);

    assignment.var.tpe.set(Some(assignment.expr.get_type()))
}

fn link_static_method_invoke(
    invoke: &tree::Invoke,
    class: &tree::Class,
    scope: &mut scope::Scope,
) {
    {
        let f = match scope.read_static_method(&class.name, &invoke.name) {
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
    context: &Context
) -> tree::Mod {
    let mut vec = Vec::new();

    for unit in &(*m).units {
        vec.push(build_mod_unit(unit, context));
    }

    tree::Mod {
        units: vec,
        number_class: Cell::new(None),
        llvm_number_class: Cell::new(None),
        boolean_class: Cell::new(None),
        llvm_boolean_class: Cell::new(None),
        string_class: Cell::new(None),
        llvm_string_class: Cell::new(None),
        array_class: Cell::new(None),
        llvm_array_class: Cell::new(None),
        char_class: Cell::new(None),
        llvm_char_class: Cell::new(None),
    }
}

fn build_mod_unit(
    unit: &syntax::tree::ModUnit,
    context: &Context
) -> tree::ModUnit {
    match unit {
        syntax::tree::ModUnit::Func(ref func) => tree::ModUnit::Func(Box::new(build_func(func, context))),
        syntax::tree::ModUnit::Class(ref class) => tree::ModUnit::Class(Box::new(build_class(class, context))),
    }
}

fn build_class(
    class: &syntax::tree::Class,
    context: &Context,
) -> tree::Class {
    let mut param_vec = vec![];
    for param in &class.params {
        param_vec.push(Box::new(build_param(&param, context)));
    }

    let mut extend_vec = vec![];
    for extend in &class.extends {
        extend_vec.push(extend.to_string());
    }

    let mut method_vec = vec![];
    for method in &class.methods {
        method_vec.push(build_method(&method, class, context));
    }

    tree::Class {
        name: class.name.to_string(),
        params: param_vec,
        extends: extend_vec,
        methods: method_vec,
        is_llvm: class.is_llvm,
        llvm_struct_type_ref: Cell::new(None),
    }
}

fn build_param(
    param: &syntax::tree::Param,
    context: &Context,
) -> tree::Param {
    tree::Param {
        var: Box::new(build_var(&param.var, context)),
        tpe_name: param.tpe.to_string(),
        tpe: Cell::new(None),
    }
}

fn build_method(
    method: &syntax::tree::Func,
    class: &syntax::tree::Class,
    context: &Context,
) -> tree::Func {
    let mut params = vec![];

    if !method.is_static {
        params.push(tree::Param {
            var: Box::new(tree::Var {
                llvm_ref: Cell::new(None),
                tpe: Cell::new(None),
                name: "__self".to_string(),
            }),
            tpe_name: class.name.to_string(),
            tpe: Cell::new(None),
        });
    }
    for param in &method.params {
        params.push(build_param(param, context))
    }

    let mut exprs = vec![];
    for expr in &method.exprs {
        exprs.push(build_expr(expr, context))
    }

    tree::Func {
        llvm_ref: Cell::new(None),
        parent_class_opt: Cell::new(None),
        name: method.name.to_string(),
        is_static: method.is_static,
        params,
        return_type_name: method.return_type.to_string(),
        return_type: Cell::new(None),
        exprs,
    }
}

fn build_func(
    func: &syntax::tree::Func,
    context: &Context,
) -> tree::Func {
    let mut params = vec![];
    for param in &func.params {
       params.push(build_param(param, context))
    }

    let mut exprs = vec![];
    for expr in &func.exprs {
        exprs.push(build_expr(expr, context))
    }

    tree::Func {
        llvm_ref: Cell::new(None),
        parent_class_opt: Cell::new(None),
        name: func.name.to_string(),
        is_static: false,
        params,
        return_type_name: func.return_type.to_string(),
        return_type: Cell::new(None),
        exprs,
    }
}

fn build_expr(
    expr: &syntax::tree::Expr,
    context: &Context,
) -> tree::Expr {
    match *expr {
        syntax::tree::Expr::Invoke(ref i) => tree::Expr::Invoke(Box::new(build_invoke(i, context))),
        syntax::tree::Expr::LlvmInvoke(ref i) => tree::Expr::LlvmInvoke(Box::new(build_llvm_invoke(i, context))),
        syntax::tree::Expr::Assignment(ref a) => tree::Expr::Assignment(Box::new(build_assignment(a, context))),
        syntax::tree::Expr::Reassignment(ref a) => tree::Expr::Reassignment(Box::new(build_reassignment(a, context))),
        syntax::tree::Expr::Var(ref v) => tree::Expr::ReadVar(Box::new(build_read_var(v, context))),
        syntax::tree::Expr::IfElse(ref if_else) => tree::Expr::IfElse(Box::new(build_if_else(if_else, context))),
        syntax::tree::Expr::ClassInstance(ref class_instance) => tree::Expr::ClassInstance(Box::new(build_class_instance(class_instance, context))),
        syntax::tree::Expr::LlvmClassInstance(ref class_instance) => tree::Expr::LlvmClassInstance(Box::new(build_llvm_class_instance(class_instance, context))),
        syntax::tree::Expr::DotInvoke(ref dot_invoke) => tree::Expr::DotInvoke(Box::new(build_dot_invoke(dot_invoke, context))),
        syntax::tree::Expr::DotMember(ref dot_member) => tree::Expr::DotMember(Box::new(build_dot_member(dot_member, context))),
        syntax::tree::Expr::LiteralString(ref s) => build_literal_string(s, context),
        syntax::tree::Expr::Num(ref num) => build_num(num, context),
        syntax::tree::Expr::Boolean(ref b) => build_boolean(b, context),
        syntax::tree::Expr::Array(ref a) => build_array(a, context),
        syntax::tree::Expr::Char(ref c) => build_char(c, context),
        syntax::tree::Expr::While(ref a) => tree::Expr::While(Box::new(build_while(a, context))),
        syntax::tree::Expr::StaticClassInstance(ref s) => tree::Expr::StaticClassInstance(Box::new(build_static_class_instance(s, context))),
    }
}

fn build_static_class_instance(
    instance: &syntax::tree::StaticClassInstance,
    context: &Context
) -> tree::StaticClassInstance {
    tree::StaticClassInstance {
        name: instance.name.to_string(),
        class_ref: Cell::new(None),
        tpe: Cell::new(None)
    }
}

fn build_while(
    whle: &syntax::tree::While,
    context: &Context
) -> tree::While {
    let mut exprs: Vec<Box<tree::Expr>> = vec![];

    for expr in &whle.exprs {
       exprs.push(Box::new(build_expr(expr, context)));
    }

    tree::While {
        cond: Box::new(build_expr(&whle.cond, context)),
        exprs,
        tpe: Cell::new(Some(tree::ExprType::Void))
    }
}

fn build_dot_member(
    dot_member: &syntax::tree::DotMember,
    context: &Context,
) -> tree::DotMember {
    tree::DotMember {
        expr: Box::new(build_expr(&dot_member.expr, context)),
        member: Box::new(build_member(&dot_member.member, context)),
        tpe: Cell::new(None),
    }
}

fn build_member(
    member: &syntax::tree::Var,
    context: &Context,
) -> tree::Member {
    tree::Member {
        name: member.name.to_string(),
        param_index: Cell::new(None),
        tpe: Cell::new(None),
    }
}

fn build_dot_invoke(
    dot_invoke: &syntax::tree::DotInvoke,
    context: &Context,
) -> tree::DotInvoke {
    tree::DotInvoke {
        expr: Box::new(build_expr(&dot_invoke.expr, context)),
        invoke: Box::new(build_invoke(&dot_invoke.invoke, context)),
        tpe: Cell::new(None),
    }
}

fn build_class_instance(
    class_instance: &syntax::tree::ClassInstance,
    context: &Context,
) -> tree::ClassInstance {
    let mut param_vec = vec![];
    for param in &class_instance.params {
       param_vec.push(Box::new(build_expr(&param, context)));
    }

    tree::ClassInstance {
        name: class_instance.name.to_string(),
        params: param_vec,
        tpe: Cell::new(None),
        class_ref: Cell::new(None),
    }
}

fn build_llvm_class_instance(
    instance: &syntax::tree::LlvmClassInstance,
    context: &Context,
) -> tree::LlvmClassInstance {
    let mut params = vec![];
    let llvm_context = Context {
        in_llvm_mode: true
    };
    for param in &instance.params {
        params.push(build_expr(&param, &llvm_context));
    }

    tree::LlvmClassInstance {
        name: instance.name.to_string(),
        params,
        tpe: Cell::new(None),
        class_ref: Cell::new(None),
    }
}

fn build_if_else(
    if_else: &syntax::tree::IfElse,
    context: &Context,
) -> tree::IfElse {
    tree::IfElse {
        cond: Box::new(build_expr(&if_else.cond, context)),
        true_br: Box::new(build_expr(&if_else.true_br, context)),
        false_br: Box::new(build_expr(&if_else.false_br, context)),
        tpe: Cell::new(None),
    }
}

fn build_read_var(
    var: &syntax::tree::Var,
    context: &Context,
) -> tree::ReadVar {
    tree::ReadVar {
        assignment_ref: Cell::new(None),
        name: var.name.to_string(),
        tpe: Cell::new(None),
        member_param_index: Cell::new(None),
    }
}

fn build_reassignment(
    reassignment: &syntax::tree::Reassignment,
    context: &Context,
) -> tree::Reassignment {
    let expr = Box::new(build_expr(&reassignment.expr, context));
    tree::Reassignment {
        var: Box::new(build_read_var(&reassignment.var, context)),
        expr,
        tpe: Cell::new(None),
    }
}

fn build_assignment(
    assignment: &syntax::tree::Assignment,
    context: &Context,
) -> tree::Assignment {
    let expr = Box::new(build_expr(&assignment.expr, context));
    tree::Assignment {
        var: Box::new(build_var(&assignment.var, context)),
        expr,
        tpe: Cell::new(None),
    }
}

fn build_var(
    var: &syntax::tree::Var,
    context: &Context,
) -> tree::Var {
    tree::Var {
        llvm_ref: Cell::new(None),
        tpe: Cell::new(None),
        name: var.name.to_string(),
    }
}

fn build_invoke(
    invoke: &syntax::tree::Invoke,
    context: &Context,
) -> tree::Invoke {
    let mut args: Vec<tree::Expr> = vec![];

    for arg in &invoke.args {
        args.push(build_expr(arg, context));
    }

    tree::Invoke {
        func_ref: Cell::new(None),
        name: invoke.name.to_string(),
        args,
        tpe: Cell::new(None),
    }
}

fn build_llvm_invoke(
    invoke: &syntax::tree::LlvmInvoke,
    context: &Context,
) -> tree::LlvmInvoke {
    let mut args: Vec<tree::Expr> = vec![];

    for arg in &invoke.args {
        args.push(build_expr(arg, context));
    }

    tree::LlvmInvoke {
        name: invoke.name.to_string(),
        is_varargs: invoke.is_varargs,
        return_type_name: invoke.return_type.to_string(),
        return_type: Cell::new(None),
        args,
    }
}

fn build_literal_string(
    literal_string: &syntax::tree::LiteralString,
    context: &Context,
) -> tree::Expr {
    let llvm_string = tree::Expr::LlvmString(Box::new(tree::LlvmString { value: literal_string.content.to_string() }));
    if context.in_llvm_mode {
        llvm_string
    } else {
        let mut items: Vec<Box<syntax::tree::Expr>> = vec![];
        for c in literal_string.content.chars() {
            items.push(Box::new(syntax::tree::Expr::ClassInstance(Box::new(
                syntax::tree::ClassInstance {
                    name: "Char".to_string(),
                    params: vec![
                        Box::new(syntax::tree::Expr::LlvmClassInstance(Box::new(
                            syntax::tree::LlvmClassInstance {
                                name: "@Char".to_string(),
                                params: vec![
                                    Box::new(syntax::tree::Expr::Char(Box::new(
                                        syntax::tree::Char {
                                            value: c
                                        }
                                    )))
                                ]
                            }
                        )))
                    ]
                }
            ))));
        }

        let array = Box::new(build_array(
            &syntax::tree::Array {
                items
            },
            context
        ));

        tree::Expr::ClassInstance(Box::new(tree::ClassInstance {
            name: "String".to_string(),
            params: vec![array],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        }))

    }
}

fn build_char(
    char: &syntax::tree::Char,
    context: &Context,
) -> tree::Expr {
    let llvm_char = tree::Expr::LlvmChar(Box::new(tree::LlvmChar { value: char.value }));
    if context.in_llvm_mode {
        llvm_char
    } else {
        let llvm_instance = Box::new(tree::Expr::LlvmClassInstance(Box::new(tree::LlvmClassInstance {
            name: "@Char".to_string(),
            params: vec![llvm_char],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        })));

        tree::Expr::ClassInstance(Box::new(tree::ClassInstance {
            name: "Char".to_string(),
            params: vec![llvm_instance],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        }))
    }
}

fn build_num(
    num: &syntax::tree::Num,
    context: &Context,
) -> tree::Expr {
    let llvm_number = tree::Expr::LlvmNumber(Box::new(tree::LlvmNumber { value: num.value }));
    if context.in_llvm_mode {
        llvm_number
    } else {
        let llvm_instance = Box::new(tree::Expr::LlvmClassInstance(Box::new(tree::LlvmClassInstance {
            name: "@I32".to_string(),
            params: vec![llvm_number],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        })));

        tree::Expr::ClassInstance(Box::new(tree::ClassInstance {
            name: "Number".to_string(),
            params: vec![llvm_instance],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        }))
    }
}

fn build_array(
    array: &syntax::tree::Array,
    context: &Context,
) -> tree::Expr {
    let mut items: Vec<Box<tree::Expr>> = vec![];

    for item in &array.items {
       items.push(Box::new(build_expr(item, context)))
    }

    let llvm_array = tree::Expr::LlvmArray(Box::new(tree::LlvmArray { items }));
    if context.in_llvm_mode {
        llvm_array
    } else {
        let llvm_instance = Box::new(tree::Expr::LlvmClassInstance(Box::new(tree::LlvmClassInstance {
            name: "@Array".to_string(),
            params: vec![llvm_array],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        })));

        let size = Box::new(build_num(
            &syntax::tree::Num {
                value: array.items.len() as i32
            },
            context
        ));

        tree::Expr::ClassInstance(Box::new(tree::ClassInstance {
            name: "Array".to_string(),
            params: vec![llvm_instance, size],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        }))
    }
}

fn build_boolean(
    boolean: &syntax::tree::Boolean,
    context: &Context,
) -> tree::Expr {
    let llvm_boolean = tree::Expr::LlvmBoolean(Box::new(tree::LlvmBoolean { value: boolean.value }));
    if context.in_llvm_mode {
        llvm_boolean
    } else {
        let llvm_instance = Box::new(tree::Expr::LlvmClassInstance(Box::new(tree::LlvmClassInstance {
            name: "@Boolean".to_string(),
            params: vec![llvm_boolean],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        })));

        tree::Expr::ClassInstance(Box::new(tree::ClassInstance {
            name: "Boolean".to_string(),
            params: vec![llvm_instance],
            class_ref: Cell::new(None),
            tpe: Cell::new(None),
        }))
    }
}
