//! JIT compilation example for Kisumu_lang using pliron-llvm

use crate::ast::parse_program;
use crate::lowering::libc;
use crate::lowering::lower_function;
#[cfg(feature = "verbose")]
use pliron::printable::Printable;
use pliron::{
    builtin::{op_interfaces::SingleBlockRegionInterface, ops::ModuleOp},
    context::Context,
    input_error_noloc,
    op::Op,
    operation::verify_operation,
    result::Result,
};
use pliron_llvm::{
    llvm_sys::{
        core::{
            LLVMContext, LLVMModule, llvm_get_named_function, llvm_get_param_types,
            llvm_get_return_type, llvm_global_get_value_type, llvm_int_type_in_context,
        },
        target::initialize_native,
    },
    to_llvm_ir,
};

#[cfg(feature = "optimize")]
use pliron::{
    opts::{dce::dce, mem2reg::mem2reg, simplify_cfg::simplify_cfg},
    pass_manager::AnalysisManager,
};

// Lower a Kisumu_lang program to LLVM dialect and return the module operation
fn lower_to_llvm_ir(src: &str, llvm_ctx: &LLVMContext) -> Result<LLVMModule> {
    let funcs =
        parse_program(src).map_err(|e| input_error_noloc!("Failed to parse program: {}", e))?;
    let ctx = &mut Context::new();
    // TODO: Picking module name needs to vary with the input file name
    let module = ModuleOp::new(ctx, "test".try_into().expect("valid module name"));
    for func in &funcs {
        let func_op = lower_function(ctx, func)?;
        module.append_operation(ctx, func_op.get_operation(), 0);
    }

    // stdlib functions
    libc::declare_printf(ctx, &module);
    libc::declare_malloc(ctx, &module);
    libc::declare_realloc(ctx, &module);
    libc::declare_strcat(ctx, &module);
    libc::declare_strncpy(ctx, &module);
    libc::declare_strcpy(ctx, &module);
    libc::declare_strndup(ctx, &module);
    libc::declare_exit(ctx, &module);
    libc::declare_free(ctx, &module);

    crate::lowering::klir::lower_module(ctx, module)?;

    verify_operation(module.get_operation(), ctx)?;

    #[cfg(feature = "optimize")]
    {
        fn optimize(module: ModuleOp, ctx: &mut Context) -> Result<()> {
            let mut manager = AnalysisManager::default();
            let _ = simplify_cfg(module.get_operation(), ctx);
            let _ = dce(module.get_operation(), ctx);
            let _ = mem2reg(module.get_operation(), ctx, &mut manager);
            Ok(())
        }

        optimize(module, ctx)?;
    }

    #[cfg(feature = "verbose")]
    println!("Pliron IR\n{}\n", module.get_operation().disp(ctx));

    // Convert from LLVM dialect to LLVM IR
    let llvm_module = to_llvm_ir::convert_module(ctx, llvm_ctx, module)?;
    llvm_module
        .verify()
        .map_err(|e| input_error_noloc!("Generated LLVM module is invalid: {}", e))?;
    Ok(llvm_module)
}

/// Execute the function `name` of a Kisumu_lang program using JIT compilation
/// The function must have the signature `fn(i64) -> i64` -- NOTE: This is subject to change
pub fn exec_fn(src: &str, name: &str, arg: i64) -> Result<(i64, String)> {
    initialize_native()
        .map_err(|e| input_error_noloc!("Failed to initialize native target: {}", e))?;
    let llvm_ctx = LLVMContext::default();
    let llvm_module = lower_to_llvm_ir(src, &llvm_ctx)?;

    let Some(f) = llvm_get_named_function(&llvm_module, name) else {
        return Err(input_error_noloc!(
            "Function '{}' not found in generated LLVM module",
            name
        ));
    };
    let f_ty = llvm_global_get_value_type(f);
    let param_types = llvm_get_param_types(f_ty);
    let ret_type = llvm_get_return_type(f_ty);
    let llvm_int64_ty = llvm_int_type_in_context(&llvm_ctx, 64);
    if param_types.len() != 1 || param_types[0] != llvm_int64_ty || ret_type != llvm_int64_ty {
        return Err(input_error_noloc!(
            "Expected function '{}' to have exactly one parameter of type i64 and return type i64, but found different signature",
            name
        ));
    }
    let llvm_out = llvm_module.to_string().clone();
    // println!("Generated LLVM IR:\n{}", llvm_module.to_string());

    // JIT compile and execute the main function
    let lljit = pliron_llvm::llvm_sys::lljit::LLVMLLJIT::new_with_default_builder()
        .map_err(|e| input_error_noloc!("Failed to create JIT execution engine: {}", e))?;
    lljit
        .add_module(llvm_module)
        .map_err(|e| input_error_noloc!("Failed to add module to JIT: {}", e))?;
    let main_fn = lljit
        .lookup_symbol(name)
        .map_err(|e| input_error_noloc!("Failed to find main function in JIT: {}", e))?;

    let main_fn: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(main_fn) };
    Ok((main_fn(arg), llvm_out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_jit() {
        let src = std::fs::read_to_string("./src/bin/klc/examples/fibonacci.kl")
            .expect("failed to read fibonacci.kl");
        let (result, _) = exec_fn(&src, "main", 5).expect("failed to execute main function");
        assert_eq!(result, 5);
    }

    #[test]
    fn factorial_jit() {
        let src = std::fs::read_to_string("./src/bin/klc/examples/factorial.kl")
            .expect("failed to read factorial.kl");
        let (result, _) = exec_fn(&src, "main", 5).expect("failed to execute main function");
        assert_eq!(result, 120);
    }

    #[test]
    fn if_else_jit() {
        let src = "
            func abs(x) {
                const result = 0;
                if x < 0 {
                    result = 0 - x;
                } else {
                    result = x;
                }
                return result;
            }
        ";
        let (result, _) = exec_fn(src, "abs", 42).expect("failed to execute main function");
        assert_eq!(result, 42);
        let (result, _) = exec_fn(src, "abs", -42).expect("failed to execute main function");
        assert_eq!(result, 42);
        let (result, _) = exec_fn(src, "abs", 0).expect("failed to execute main function");
        assert_eq!(result, 0);
    }
}
