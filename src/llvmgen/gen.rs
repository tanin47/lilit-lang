use inkwell::AddressSpace;
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

struct Core {
   string_struct_type: StructType
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

    for unit in &module.units {
        gen_mod_unit(unit, &llvm_module, context, builder, &core);
    }
    return llvm_module;
}

fn gen_mod_unit(
    unit: &tree::ModUnit,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) {
    match unit {
        tree::ModUnit::Func { ref func } => {
            gen_func(func, module, context, builder, core);
        },
        _ => (),
    }
}

fn gen_func(
    func: &tree::Func,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function(&func.name, fn_type, None);
    func.llvm_ref.replace(Some(function));

    for (index, expr) in func.exprs.iter().enumerate() {
        let basic_block = context.append_basic_block(&function, &format!("block_{}", index));
        if index > 0 {
            builder.build_unconditional_branch(&basic_block);
        }

        builder.position_at_end(&basic_block);

        let ret = gen_expr(expr, module, context, builder, core);

        if index == (func.exprs.len() - 1) {
            builder.build_return(Some(&ret));
        }
    }
}

fn gen_expr(
    expr: &tree::Expr,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    match expr {
        tree::Expr::Invoke { ref invoke } => {
            gen_invoke(invoke, module, context, builder, core).into()
        },
        tree::Expr::Num { ref num } => {
            gen_num(num, module, context, builder, core).into()
        },
        tree::Expr::Assignment { ref assignment } => {
            gen_assignment(assignment, module, context, builder, core).into()
        },
        tree::Expr::ReadVar { ref read_var } => {
            gen_read_var(read_var, module, context, builder, core).into()
        },
        tree::Expr::LiteralString { ref literal_string } => {
            gen_string(literal_string, module, context, builder, core).into()
        },
    }
}

fn gen_string(
    literal_string: &tree::LiteralString,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> PointerValue {
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();

    let string = builder.build_alloca(core.string_struct_type, "string");

    let array_type = i8_type.array_type((literal_string.content.len() + 1) as u32);
    let array = builder.build_alloca(array_type, "string_array");

    for (index, c) in literal_string.content.chars().enumerate() {
        let p = unsafe {
            builder.build_in_bounds_gep(
                array,
                &[i32_type.const_int(0, false), i32_type.const_int(index as u64, false)],
                "gep")
        };
        builder.build_store(p, i8_type.const_int(c as u64, false));
    }
    // Store string terminating symbol
    let last = unsafe {
        builder.build_in_bounds_gep(
            array,
            &[i32_type.const_int(0, false), i32_type.const_int(literal_string.content.len() as u64, false)],
            "gep")
    };
    builder.build_store(last, i8_type.const_int(0, false));

    let size = i32_type.const_int((literal_string.content.len() + 1) as u64, false);

    let size_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    builder.build_store(size_pointer, size);

    let content_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    builder.build_store(content_pointer, array);

    string
}

fn gen_string_from_cstring(
    cstring: PointerValue,
    cstring_size: IntValue,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> PointerValue {
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();

    let string = builder.build_alloca(core.string_struct_type, "string");

    let size_with_terminator = cstring_size.const_add(context.i32_type().const_int(1, false));
    let array = builder.build_array_alloca(i8_type, size_with_terminator,  "string_array");

    let memcpy = match module.get_function("llvm.memcpy.p0i8.p0i8.i64") {
        None => {
           module.add_function(
               "llvm.memcpy.p0i8.p0i8.i64",
               context.i64_type().fn_type(
                   &[
                       i8_type.ptr_type(AddressSpace::Generic).into(),
                       i8_type.ptr_type(AddressSpace::Generic).into(),
                       context.i64_type().into(),
                       context.i32_type().into(),
                       context.bool_type().into()
                   ],
                   false
               ),
               Some(Linkage::External)
           )
        }
        Some(f) => f,
    };

    builder.build_call(
        memcpy,
        &[
            array.into(),
            cstring.into(),
            size_with_terminator.into(),
            context.i32_type().const_int(4, false).into(),
            context.bool_type().const_zero().into()
        ],
        "memcpy"
    );

    let size_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(0, false)],
            "gep"
        )
    };
    builder.build_store(size_pointer, cstring_size);

    let content_pointer = unsafe {
        builder.build_in_bounds_gep(
            string,
            &[i32_type.const_int(0, false), i32_type.const_int(1, false)],
            "gep"
        )
    };
    builder.build_store(content_pointer, array);

    string
}

fn gen_read_var(
    var: &tree::ReadVar,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    let assignment = unsafe { &*var.assignment_ref.get().unwrap() };
    let i32_type = context.i32_type();
    let value = builder.build_load(
        assignment.llvm_ref.get().unwrap(),
        &var.name
    );
    value.into()
}

fn gen_assignment(
    assignment: &tree::Assignment,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    let expr = gen_expr(&assignment.expr, module, context, builder, core);

   let ptr = match expr {
       BasicValueEnum::IntValue(_) => {
           let i32_type = context.i32_type();
           builder.build_alloca(i32_type, &assignment.var.name)
       },
       BasicValueEnum::PointerValue(p) => {
           let ptr_type = core.string_struct_type.ptr_type(AddressSpace::Generic);
           builder.build_alloca(ptr_type, &assignment.var.name)
       },
       _ => panic!("Unknow expr")
   } ;


    builder.build_store(ptr, expr);
    assignment.var.llvm_ref.replace(Some(ptr));
    expr.into()
}

fn gen_num(
    num: &tree::Num,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> IntValue {
    let i32_type = context.i32_type();
    i32_type.const_int(num.value as u64, false)
}

fn gen_invoke(
    invoke: &tree::Invoke,
    module: &Module,
    context: &Context,
    builder: &Builder,
    core: &Core,
) -> BasicValueEnum {
    if invoke.name == "print" {
        let printf = match module.get_function("printf") {
            Some(f) => f,
            None => {
                let str_type = context.i8_type().ptr_type(AddressSpace::Generic);
                let i32_type = context.i32_type();
                let printf_type = i32_type.fn_type(&[str_type.into()], true);
                module.add_function("printf", printf_type, Some(Linkage::External))
            },
        };

        let i32_type = context.i32_type();
        let ptr_type = context.i32_type().ptr_type(AddressSpace::Generic);
        let arg = gen_expr(&invoke.arg, module, context, builder, core);
        let ss = match arg {
            BasicValueEnum::PointerValue(ptr) => ptr,
            _ => panic!("fail arg"),
        };
        let s = unsafe {
            builder.build_in_bounds_gep(ss, &[i32_type.const_int(0, false), i32_type.const_int(1, false)], "")
        };
        let l = builder.build_load(s, "load");
        builder.build_call(printf, &[l], "").try_as_basic_value().left().unwrap().into()
    } else if invoke.name == "read" {
        let io_struct = context.opaque_struct_type("struct._IO_FILE");
        let fgets = match module.get_function("fgets") {
            Some(f) => f,
            None => {
                let fgets_type = context.i8_type().ptr_type(AddressSpace::Generic).fn_type(
                    &[
                        context.i8_type().ptr_type(AddressSpace::Generic).into(),
                        context.i32_type().into(),
                        io_struct.ptr_type(AddressSpace::Generic).into(),
                    ],
                    false);
                module.add_function("fgets", fgets_type, Some(Linkage::External))
            },
        };
        let stdin = match module.get_global("stdin") {
            None => {
                let g = module.add_global(
                    io_struct.ptr_type(AddressSpace::Generic),
                    None,
                    "stdin");
                g
            },
            Some(g) => g
        };

        let input_size = 100;
        let input = builder.build_alloca(context.i8_type().array_type(input_size), "input");
        builder.build_call(
            fgets,
            &[
                input.into(),
                context.i32_type().const_int(input_size as u64, false).into(),
                builder.build_load(stdin.as_pointer_value(), "load_stdin"),
            ],
            "fgets").try_as_basic_value().left().unwrap();
        let strlen = match module.get_function("strlen") {
            Some(f) => f,
            None => {
                let fn_type = context.i64_type().fn_type(
                    &[
                        context.i8_type().ptr_type(AddressSpace::Generic).into()
                    ],
                    false);
                module.add_function("strlen", fn_type, Some(Linkage::External))
            },
        };
        let ret_str_len = builder.build_call(strlen, &[input.into()], "strlen");
        let size = match ret_str_len.try_as_basic_value().left().unwrap() {
            BasicValueEnum::IntValue(i) => i,
            _ => panic!("unable to get string's length")
        };
        gen_string_from_cstring(input, size, module, context, builder, core).into()
    } else {
        let func = unsafe { &*invoke.func_ref.get().unwrap() };
        builder.build_call(func.llvm_ref.get().unwrap(), &[], &invoke.name).try_as_basic_value().left().unwrap().into()
    }
}
