pub mod libc {
    // use crate::dialect::{
    //     BinOp, BinOpKind, CallOp as KalCallOp, ConstantOp as KalConstantOp, DeclOp as KalDeclOp,
    //     IfOp as KalIfOp, LoadOp as KalLoadOp, ReturnOp as KalReturnOp, StoreOp as KalStoreOp,
    //     StringOp as KalStringOp, WhileOp as KalWhileOp, YieldOp as KalYieldOp,
    // };
    use pliron::builtin::op_interfaces::SingleBlockRegionInterface;
    use pliron::{
        builtin::{
            ops::ModuleOp,
            types::{FunctionType, IntegerType, Signedness},
        },
        context::Context,
        op::Op,
    };
    use pliron_llvm::types::{PointerType, VoidType};

    // libc-decl functions
    pub fn declare_printf(ctx: &mut Context, module: &ModuleOp) {
        let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
        let ptr_ty = PointerType::get(ctx, 0);
        let func_ty = FunctionType::get(ctx, vec![ptr_ty.into()], vec![i32_ty.into()]);

        let name = "printf".try_into().expect("valid identifier");
        let printf_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);

        // Ensure it is a *declaration* (no body), not a definition.
        // If your FuncOp starts as a definition with an empty block, mark it
        // external / declaration according to your pliron version, e.g.:
        //   printf_decl.set_declaration(ctx);
        // or simply never add a body / terminator.

        module.append_operation(ctx, printf_decl.get_operation(), 0);
    }

    pub fn declare_malloc(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        let func_ty = FunctionType::get(ctx, vec![ptr_ty.into()], vec![i8_ptr_ty.into()]);
        let name = "malloc".try_into().expect("valid identifier");
        let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, malloc_decl.get_operation(), 0);
    }

    pub fn declare_realloc(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let func_ty = FunctionType::get(
            ctx,
            vec![ptr_ty.into(), i64_ty.into()],
            vec![i8_ptr_ty.into()],
        );
        let name = "realloc".try_into().expect("valid identifier");
        let realloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, realloc_decl.get_operation(), 0);
    }

    pub fn declare_strcat(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        // let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let func_ty = FunctionType::get(
            ctx,
            vec![ptr_ty.clone().into(), ptr_ty.into()],
            vec![i8_ptr_ty.into()],
        );
        let name = "strcat".try_into().expect("valid identifier");
        let strcat_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, strcat_decl.get_operation(), 0);
    }

    pub fn declare_strcpy(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        let func_ty = FunctionType::get(
            ctx,
            vec![ptr_ty.clone().into(), ptr_ty.into()],
            vec![i8_ptr_ty.into()],
        );
        let name = "strcpy".try_into().expect("valid identifier");
        let strcpy_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, strcpy_decl.get_operation(), 0);
    }

    pub fn declare_strncpy(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let func_ty = FunctionType::get(
            ctx,
            vec![ptr_ty.clone().into(), ptr_ty.into(), i64_ty.into()],
            vec![i8_ptr_ty.into()],
        );
        let name = "strncpy".try_into().expect("valid identifier");
        let strncpy_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, strncpy_decl.get_operation(), 0);
    }

    pub fn declare_strndup(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let ptr_ty = PointerType::get(ctx, 0);
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let func_ty = FunctionType::get(
            ctx,
            vec![ptr_ty.into(), i64_ty.into()],
            vec![i8_ptr_ty.into()],
        );
        let name = "strndup".try_into().expect("valid identifier");
        let strndup_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, strndup_decl.get_operation(), 0);
    }

    pub fn declare_free(ctx: &mut Context, module: &ModuleOp) {
        let void_ty = VoidType::get(ctx);
        let ptr_ty = PointerType::get(ctx, 0);
        let func_ty = FunctionType::get(ctx, vec![ptr_ty.into()], vec![void_ty.into()]);
        let name = "free".try_into().expect("valid identifier");
        let free_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, free_decl.get_operation(), 0);
    }

    pub fn declare_exit(ctx: &mut Context, module: &ModuleOp) {
        let i8_ptr_ty = VoidType::get(ctx);
        let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
        let func_ty = FunctionType::get(ctx, vec![i32_ty.into()], vec![i8_ptr_ty.into()]);
        let name = "exit".try_into().expect("valid identifier");
        let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
        module.append_operation(ctx, malloc_decl.get_operation(), 0);
    }
}
