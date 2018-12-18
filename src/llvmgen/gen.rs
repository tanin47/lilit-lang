use inkwell::AddressSpace;
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::ArrayType;
use inkwell::types::BasicTypeEnum;
use inkwell::types::StructType;
use inkwell::values::{BasicValue, FunctionValue, IntValue};
use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;
use inkwell::values::StructValue;
use inkwell::values::VectorValue;

use semantics::tree;
use inkwell::types::PointerType;

#[derive(Debug)]
enum Value {
    Void,
    Number(IntValue),
    Boolean(IntValue),
    String(PointerValue),
    Class(PointerValue, *const tree::ClassInstance),
    LlvmClass(PointerValue, *const tree::LlvmClassInstance),
}

fn convert(value: &Value) -> BasicValueEnum {
    match value {
        Value::Number(i) => (*i).into(),
        Value::Boolean(b) => (*b).into(),
        Value::String(p) => (*p).into(),
        Value::Class(p, c) => (*p).into(),
        Value::LlvmClass(p, c) => (*p).into(),
        Value::Void => panic!("can't convert void"),
    }
}

struct Core {
    string_struct_type: StructType
}

struct ModContext<'a, 'b, 'c, 'd> {
    module: &'a Module,
    context: &'b Context,
    builder: &'c Builder,
    core: &'d Core,
}

struct FnContext<'a, 'b, 'c, 'd, 'e> {
    func: &'a FunctionValue,
    module: &'b Module,
    context: &'c Context,
    builder: &'d Builder,
    core: &'e Core,
}

pub fn generate(
    module: &tree::Mod,
    context: &Context,
    builder: &Builder,
) -> Module {
    let llvm_module = context.create_module("main");

    let core = Core {
        string_struct_type: StructType::struct_type(
            &[
                context.i32_type().into(),
                context.i8_type().ptr_type(AddressSpace::Generic).into()
            ],
            false
        ),
    };
    {
        let context = ModContext {
            module: &llvm_module,
            context,
            builder,
            core: &core
        };
        for unit in &module.units {
            gen_mod_unit(unit, &context);
        }
    }
    return llvm_module;
}

fn gen_mod_unit(
    unit: &tree::ModUnit,
    context: &ModContext,
) {
    match unit {
        tree::ModUnit::Func { ref func } => {
            gen_func(func, context);
        },
        _ => (),
    }
}

fn gen_func(
    func: &tree::Func,
    context: &ModContext,
) {
    let i32_type = context.context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = context.module.add_function(&func.name, fn_type, None);
    func.llvm_ref.replace(Some(function));

    let first_block = context.context.append_basic_block(&function, "first_block");
    context.builder.position_at_end(&first_block);

    let fn_context = FnContext {
        func: &function,
        module: context.module,
        context: context.context,
        builder: context.builder,
        core: context.core,
    };

    for (index, expr) in func.exprs.iter().enumerate() {
        let ret = gen_expr(expr, &fn_context);
        if index == (func.exprs.len() - 1) {
            context.builder.build_return(Some(&convert(&ret)));
        }
    }
}

fn gen_expr(
    expr: &tree::Expr,
    context: &FnContext
) -> Value {
    match expr {
        tree::Expr::Invoke { ref invoke, tpe: _ } => {
            gen_invoke(invoke, context)
        },
        tree::Expr::LlvmInvoke { ref invoke, tpe: _ } => {
            gen_llvm_invoke(invoke, context)
        },
        tree::Expr::Num { ref num, tpe: _ } => {
            gen_num(num, context)
        },
        tree::Expr::Assignment { ref assignment, tpe: _ } => {
            gen_assignment(assignment, context)
        },
        tree::Expr::ReadVar { ref read_var, tpe: _ } => {
            gen_read_var(read_var, context)
        },
        tree::Expr::LiteralString { ref literal_string, tpe: _ } => {
            gen_string(literal_string, context)
        },
        tree::Expr::Boolean { ref boolean, tpe: _ } => {
            gen_boolean(boolean, context)
        },
        tree::Expr::Comparison { ref comparison, tpe: _ } => {
            gen_comparison(comparison, context)
        },
        tree::Expr::IfElse { ref if_else, tpe: _ } => {
            gen_if_else(if_else, context)
        },
        tree::Expr::ClassInstance { ref class_instance, tpe: _ } => {
            gen_class_instance(class_instance, context)
        },
        tree::Expr::LlvmClassInstance { ref class_instance, tpe: _ } => {
            gen_llvm_class_instance(class_instance, context)
        },
    }
}

fn gen_class_instance(
    class_instance: &tree::ClassInstance,
    context: &FnContext
) -> Value {
    let s = match gen_expr(&class_instance.expr, context) {
        Value::String(ptr) => ptr,
        _ => panic!("A class expects one string as its parameter"),
    };
    let class_struct = StructType::struct_type(
        &[
            s.get_type().into()
        ],
        false
    );
    let instance= context.builder.build_alloca(class_struct, "class");
    let first_param = unsafe {
        context.builder.build_in_bounds_gep(
            instance,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
            "gep")
    };
    context.builder.build_store(first_param, s);

    Value::Class(instance, class_instance as *const tree::ClassInstance)
}

fn gen_llvm_class_instance(
    class_instance: &tree::LlvmClassInstance,
    context: &FnContext
) -> Value {
    let value = gen_expr(&class_instance.expr, context);
    let tpe: BasicTypeEnum = match value {
        Value::String(ptr) => ptr.get_type().into(),
        Value::LlvmClass(ptr, _) => ptr.get_type().into(),
        Value::Number(i) => i.get_type().into(),
        _ => panic!("A class expects one string as its parameter"),
    };
    let class_struct = StructType::struct_type(&[tpe], false);
    let instance= context.builder.build_alloca(class_struct, "class");
    let first_param = unsafe {
        context.builder.build_in_bounds_gep(
            instance,
            &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
            "gep")
    };
    context.builder.build_store(first_param, convert(&value));

    Value::LlvmClass(instance, class_instance as *const tree::LlvmClassInstance)
}

fn gen_boolean(
    boolean: &tree::Boolean,
    context: &FnContext,
) -> Value {
   Value::Boolean(context.context.bool_type().const_int(boolean.value as u64, false))
}

fn gen_comparison(
    comparison: &tree::Comparison,
    context: &FnContext,
) -> Value {
    let var = match gen_read_var(&comparison.left, context) {
        Value::Number(i) => i,
        _ => panic!("Unable to read var into IntValue")
    };
    let num = match gen_num(&comparison.right, context) {
        Value::Number(i) => i,
        _ => panic!("")
    };
    Value::Boolean(context.builder.build_int_compare(IntPredicate::SGT, var, num, "cond"))
}

fn gen_if_else(
    if_else: &tree::IfElse,
    context: &FnContext,
) -> Value {
    let comparison = match gen_comparison(&if_else.cond, context) {
        Value::Boolean(i) => i,
        _ => panic!(""),
    };
    let true_block = context.context.append_basic_block(context.func, "true_block");
    let false_block = context.context.append_basic_block(context.func, "false_block");
    let end_block = context.context.append_basic_block(context.func, "end");
    context.builder.build_conditional_branch(comparison, &true_block, &false_block);

    context.builder.position_at_end(&true_block);
    let true_value = gen_expr(&if_else.true_br, context);

    context.builder.position_at_end(&false_block);
    let false_value = gen_expr(&if_else.false_br, context);

    match (&true_value, &false_value) {
        (Value::Number(_), Value::Number(_)) => {
            let ret_pointer = context.builder.build_alloca(context.context.i32_type(), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::IntValue(i) => Value::Number(i),
                _ => panic!("")
            }
        },
        (Value::String(_), Value::String(_)) => {
            let ret_pointer = context.builder.build_alloca(context.core.string_struct_type.ptr_type(AddressSpace::Generic), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::PointerValue(i) => Value::String(i),
                _ => panic!("")
            }
        },
        (Value::Boolean(_), Value::Boolean(_)) => {
            let ret_pointer = context.builder.build_alloca(context.context.i32_type(), "ret_if_else");

            context.builder.position_at_end(&true_block);
            context.builder.build_store(ret_pointer, convert(&true_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&false_block);
            context.builder.build_store(ret_pointer, convert(&false_value));
            context.builder.build_unconditional_branch(&end_block);

            context.builder.position_at_end(&end_block);
            match context.builder.build_load(ret_pointer, "load_ret_if_else") {
                BasicValueEnum::IntValue(i) => Value::Boolean(i),
                _ => panic!("")
            }
        },
        (Value::Void, _) => Value::Void,
        (_, Value::Void) => Value::Void,
        _ => panic!("")
    }
}

fn gen_string(
    literal_string: &tree::LiteralString,
    context: &FnContext
) -> Value {
    let i8_type = context.context.i8_type();
    let i32_type = context.context.i32_type();

    let string = context.builder.build_alloca(context.core.string_struct_type, "string");

    let array_type = i8_type.array_type((literal_string.content.len() + 1) as u32);
    let array = context.builder.build_alloca(array_type, "string_array");

    for (index, c) in literal_string.content.chars().enumerate() {
        let p = unsafe {
            context.builder.build_in_bounds_gep(
                array,
                &[i32_type.const_int(0, false), i32_type.const_int(index as u64, false)],
                "gep")
        };
        context.builder.build_store(p, i8_type.const_int(c as u64, false));
    }
    // Store string terminating symbol
    let last = unsafe {
        context.builder.build_in_bounds_gep(
            array,
            &[i32_type.const_int(0, false), i32_type.const_int(literal_string.content.len() as u64, false)],
            "gep")
    };
    context.builder.build_store(last, i8_type.const_int(0, false));

    let size = i32_type.const_int((literal_string.content.len() + 1) as u64, false);

    let size_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    context.builder.build_store(size_pointer, size);

    let content_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    context.builder.build_store(content_pointer, array);

    Value::String(string)
}

fn gen_string_from_cstring(
    cstring: PointerValue,
    context: &FnContext
) -> Value {
    let strlen = match context.module.get_function("strlen") {
        Some(f) => f,
        None => {
            let fn_type = context.context.i64_type().fn_type(
                &[
                    context.context.i8_type().ptr_type(AddressSpace::Generic).into()
                ],
                false);
            context.module.add_function("strlen", fn_type, Some(Linkage::External))
        },
    };
    let ret_strlen = context.builder.build_call(strlen, &[cstring.into()], "strlen");
    let cstring_size = match ret_strlen.try_as_basic_value().left().unwrap() {
        BasicValueEnum::IntValue(i) => i,
        _ => panic!("unable to get string's length")
    };

    let i8_type = context.context.i8_type();
    let i32_type = context.context.i32_type();

    let string = context.builder.build_alloca(context.core.string_struct_type, "string");

    let size_with_terminator = cstring_size.const_add(context.context.i32_type().const_int(1, false));
    let array = context.builder.build_array_alloca(i8_type, size_with_terminator,  "string_array");

    let memcpy = match context.module.get_function("llvm.memcpy.p0i8.p0i8.i64") {
        None => {
           context.module.add_function(
               "llvm.memcpy.p0i8.p0i8.i64",
               context.context.i64_type().fn_type(
                   &[
                       i8_type.ptr_type(AddressSpace::Generic).into(),
                       i8_type.ptr_type(AddressSpace::Generic).into(),
                       context.context.i64_type().into(),
                       context.context.i32_type().into(),
                       context.context.bool_type().into()
                   ],
                   false
               ),
               Some(Linkage::External)
           )
        }
        Some(f) => f,
    };

    context.builder.build_call(
        memcpy,
        &[
            array.into(),
            cstring.into(),
            size_with_terminator.into(),
            context.context.i32_type().const_int(4, false).into(),
            context.context.bool_type().const_zero().into()
        ],
        "memcpy"
    );

    let size_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    context.builder.build_store(size_pointer, cstring_size);

    let content_pointer = unsafe {
        context.builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    context.builder.build_store(content_pointer, array);

    Value::String(string)
}

fn gen_read_var(
    var: &tree::ReadVar,
    context: &FnContext,
) -> Value {
    let assignment = unsafe { &*var.assignment_ref.get().unwrap() };
    let i32_type = context.context.i32_type();
    let value = context.builder.build_load(
        assignment.llvm_ref.get().unwrap(),
        &var.name
    );
    match value {
        BasicValueEnum::IntValue(i) => Value::Number(i),
        BasicValueEnum::PointerValue(p) => Value::String(p),
        _ => panic!("Unable to read var")
    }
}

fn gen_assignment(
    assignment: &tree::Assignment,
    context: &FnContext,
) -> Value {
    let expr = gen_expr(&assignment.expr, context);

   let ptr = match expr {
       Value::Number(_) => {
           let i32_type = context.context.i32_type();
           context.builder.build_alloca(i32_type, &assignment.var.name)
       },
       Value::String(p) => {
           let ptr_type = context.core.string_struct_type.ptr_type(AddressSpace::Generic);
           context.builder.build_alloca(ptr_type, &assignment.var.name)
       },
       _ => panic!("Unknow expr")
   } ;


    context.builder.build_store(ptr, convert(&expr));
    assignment.var.llvm_ref.replace(Some(ptr));
    Value::Void
}

fn gen_num(
    num: &tree::Num,
    context: &FnContext,
) -> Value {
    let i32_type = context.context.i32_type();
    Value::Number(i32_type.const_int(num.value as u64, false))
}

fn get_llvm_type(value: &Value, context: &FnContext) -> BasicTypeEnum {
    println!("{:?}", value);
    match value {
        Value::LlvmClass(ptr, klass_ptr) => {
            let klass = unsafe { &**klass_ptr };
            let first_param_pointer = unsafe {
                context.builder.build_in_bounds_gep(
                    *ptr,
                    &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
                    "gep")
            };
            if klass.name == "String" {
                match context.builder.build_load(first_param_pointer, "load_first_param") {
                    BasicValueEnum::PointerValue(i) => {
                        let string_pointer = unsafe {
                            context.builder.build_in_bounds_gep(
                                i,
                                &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(1, false)],
                                "gep")
                        };
                        match context.builder.build_load(string_pointer, "load_string_pointer") {
                            BasicValueEnum::PointerValue(p) => p.get_type().into(),
                            _ => panic!(""),
                        }
                    },
                    _ => panic!(""),
                }
            } else if  klass.name == "I32" || klass.name == "I8" {
                match context.builder.build_load(first_param_pointer, "load_first_param") {
                    BasicValueEnum::IntValue(i) => i.get_type().into(),
                    _ => panic!("")
                }
            } else if klass.name == "Pointer" {
                println!("load {:?}", first_param_pointer);
                match context.builder.build_load(first_param_pointer, "load_first_param") {
                    BasicValueEnum::PointerValue(p) => {
                        let inner_instance = unsafe {
                            match *klass.expr {
                                tree::Expr::LlvmClassInstance { ref class_instance, tpe: _ } => class_instance,
                                _ => panic!(""),
                            }
                        };
                        let tpe = get_llvm_type(&Value::LlvmClass(p, &**inner_instance as *const tree::LlvmClassInstance), context);
                        println!("type {:?}", tpe);
                        match tpe {
                            BasicTypeEnum::IntType(i) => i.ptr_type(AddressSpace::Generic).into(),
                            BasicTypeEnum::PointerType(p) => p.ptr_type(AddressSpace::Generic).into(),
                            _ => panic!(),
                        }
                    },
                    _ => panic!(),
                }
            } else {
                panic!()
            }
        },
        _ => panic!(),
    }
}

fn get_llvm_value(value: &Value, context: &FnContext) -> BasicValueEnum {
    match value {
        Value::LlvmClass(ptr, klass_ptr) => {
            let klass = unsafe { &**klass_ptr };
            let first_param_pointer = unsafe {
                context.builder.build_in_bounds_gep(
                    *ptr,
                    &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(0, false)],
                    "gep")
            };

            if klass.name == "String" {
                let first_param = match context.builder.build_load(first_param_pointer, "load_first_param") {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => panic!(),
                };
                let content_pointer = unsafe {
                    context.builder.build_in_bounds_gep(
                        first_param,
                        &[context.context.i32_type().const_int(0, false), context.context.i32_type().const_int(1, false)],
                        "gep"
                    )
                };

                context.builder.build_load(content_pointer, "load_string")
            } else if klass.name == "I32" || klass.name == "I8" {
                context.builder.build_load(first_param_pointer, "load_i32")
            } else if klass.name == "Pointer" {
                let first_param = match context.builder.build_load(first_param_pointer, "load_first_param") {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => panic!(),
                };
                let inner_instance = unsafe {
                    match *klass.expr {
                        tree::Expr::LlvmClassInstance { ref class_instance, tpe: _ } => class_instance,
                        _ => panic!(),
                    }
                };
                let value = get_llvm_value(&Value::LlvmClass(first_param, &**inner_instance as *const tree::LlvmClassInstance), context);
                match value {
                    BasicValueEnum::IntValue(i) => i.get_type().ptr_type(AddressSpace::Generic).const_zero().into(),
                    BasicValueEnum::PointerValue(inner) => {
                        let ptr = context.builder.build_alloca(inner.get_type().ptr_type(AddressSpace::Generic), "pointer_of_pointer");
                        context.builder.build_store(ptr, inner);
                        ptr.into()
                    }
                    _ => panic!(),
                }
            } else {
                panic!()
            }
        },
        _ => panic!(),
    }
}

fn gen_llvm_invoke(
    invoke: &tree::LlvmInvoke,
    context: &FnContext,
) -> Value {
    let mut args: Vec<Value> = vec![];
    for arg in &invoke.args {
        args.push(gen_expr(arg, context));
    }

    println!("Invoke {:?}", invoke);
    println!("Get llvm types");
    let llvm_func = match context.module.get_function(&invoke.name) {
        Some(f) => f,
        None => {
            let mut args_types: Vec<BasicTypeEnum> = vec![];
            for arg in &args {
                println!("arg {:?}", arg);
                args_types.push(get_llvm_type(arg, context))
            }
            let llvm_func_type = if invoke.return_type == "Void" {
                context.context.void_type().fn_type(
                    &args_types,
                    invoke.is_varargs
                )
            } else if invoke.return_type == "String" {
                context.context.i8_type().ptr_type(AddressSpace::Generic).fn_type(
                    &args_types,
                    invoke.is_varargs
                )
            } else if invoke.return_type == "I32" {
                context.context.i32_type().fn_type(
                    &args_types,
                    invoke.is_varargs
                )
            } else {
                panic!("")
            };
            context.module.add_function(
                &invoke.name,
                llvm_func_type,
                Some(Linkage::External)
            )
        },
    };

    println!("Get llvm args");
    let mut llvm_args: Vec<BasicValueEnum> = vec![];
    for arg in &args {
        llvm_args.push(get_llvm_value(arg, context));
    }

    let llvm_ret = context.builder.build_call(llvm_func, &llvm_args, "");

    if invoke.return_type == "Void" {
        Value::Void
    } else if invoke.return_type == "I32" {
        match llvm_ret.try_as_basic_value().left().unwrap() {
            BasicValueEnum::IntValue(i) => Value::Number(i),
            _ => panic!(),
        }
    } else if invoke.return_type == "String" {
        let p = match llvm_ret.try_as_basic_value().left().unwrap() {
            BasicValueEnum::PointerValue(ptr) => ptr,
            _ => panic!(),
        };
        gen_string_from_cstring(p, context)
    } else {
        panic!("Unrecognized llvm invoke return type")
    }
}

fn gen_invoke(
    invoke: &tree::Invoke,
    context: &FnContext,
) -> Value {
    let func = unsafe { &*invoke.func_ref.get().unwrap() };
    let llvm_ret = context.builder.build_call(func.llvm_ref.get().unwrap(), &[], &invoke.name);

    if func.return_type == "Number" {
        match llvm_ret.try_as_basic_value().left().unwrap() {
            BasicValueEnum::IntValue(i) => Value::Number(i),
            _ => panic!(""),
        }
    } else if func.return_type == "String" {
        match llvm_ret.try_as_basic_value().left().unwrap() {
            BasicValueEnum::PointerValue(p) => Value::String(p),
            _ => panic!(""),
        }
    } else if func.return_type == "Void" {
        Value::Void
    } else {
        panic!("")
    }
}
