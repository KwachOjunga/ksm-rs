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

/*
    pub mod lowering {
        use super::*;
        use pliron::builtin::op_interfaces::{CallOpCallable, OneResultInterface};
        use pliron_llvm::op_interfaces::CastOpInterface;
        use pliron::identifier::Identifier;
        // use pliron::value::Value;
        use pliron::irbuild::inserter::Inserter;
        use pliron::irbuild::{rewriter::Rewriter, dialect_conversion::DialectConversionRewriter};
        use pliron_llvm::ops::CallOp as LlvmCallOp;
        use pliron_llvm::types::FuncType;
        use pliron::r#type::TypeHandle;

        fn printf(op: &crate::dialect::CallOp, ctx: &mut Context, rewriter: &mut DialectConversionRewriter, args: Vec<pliron::value::Value>, callee_ident: Identifier, res_ty: TypeHandle) -> pliron::result::Result<()> {
            let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
            let ptr_ty = PointerType::get(ctx, 0);
            let llvm_func_ty = FuncType::get(ctx, i32_ty.into(), vec![ptr_ty.into()], true);
            let llvm_call = LlvmCallOp::new(
                ctx,
                CallOpCallable::Direct(callee_ident),
                llvm_func_ty,
                args,
            );
            let call_res = llvm_call.get_result(ctx);
            rewriter.insert_op(ctx, &llvm_call);
            let sext = SExtOp::new(ctx, call_res, res_ty);
            let final_res = sext.get_result(ctx);
            rewriter.insert_op(ctx, &sext);
            rewriter.replace_operation_with_values(
                ctx,
                op.get_operation(),
                vec![final_res],
            );
            return Ok(());
        }

        fn malloc( op: &crate::dialect::CallOp, ctx: &mut Context, rewriter: &mut DialectConversionRewriter, args:Vec<pliron::value::Value>, callee_ident: Identifier, res_ty: TypeHandle) ->  pliron::result::Result<()> {
            let i8_ty = PointerType::get(ctx, 0);
            let ptr_ty = IntegerType::get(ctx, 64, Signedness::Signless);
            let llvm_func_ty = FuncType::get(ctx, i8_ty.into(), vec![ptr_ty.into()], false);
            let llvm_call = LlvmCallOp::new(
                ctx,
                CallOpCallable::Direct(callee_ident),
                llvm_func_ty,
                args,
            );
            let call_res = llvm_call.get_result(ctx);
            rewriter.insert_op(ctx, &llvm_call);
            let sext = SExtOp::new(ctx, call_res, res_ty);
            let final_res = sext.get_result(ctx);
            rewriter.insert_op(ctx, &sext);
            rewriter.replace_operation_with_values(
                ctx,
                op.get_operation(),
                vec![final_res],
            );
            return Ok(());
        }

        pub type LibcFnPtrs<'a> = rustc_hash::FxHashMap<
            &'a str,
               fn(
                    &mut Context,
                    &mut pliron::irbuild::dialect_conversion::DialectConversionRewriter,
                    Vec<pliron::value::Value>,
                    pliron::identifier::Identifier,
                    TypeHandle
                ) -> pliron::result::Result<()>,

        >;
        pub const LibcFns : [fn(
             op: &crate::dialect::CallOp,
             &mut Context,
             &mut pliron::irbuild::dialect_conversion::DialectConversionRewriter,
             Vec<pliron::value::Value>,
             pliron::identifier::Identifier,
             TypeHandle
         ) -> pliron::result::Result<()>; 9];



        fn init_libc_fn_ptrs<'a>(
            op: &crate::dialect::CallOp,
            ctx: &mut Context,
            rewriter: &mut pliron::irbuild::dialect_conversion::DialectConversionRewriter,
            res_ty: pliron::r#type::TypeHandle,
            args: Vec<pliron::value::Value>,
        ) -> LibcFnPtrs<'a> {


            let mut map = LibcFnPtrs::default();
            map.insert(
                "printf",
                printf);
            map.insert(
                "malloc", malloc);
            map.insert(
                "realloc",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let i8_ty = PointerType::get(ctx, 0);
                    let ptr_ty = IntegerType::get(ctx, 64, Signedness::Signless);
                    let llvm_func_ty = FuncType::get(ctx, i8_ty.into(), vec![ptr_ty.into()], false);
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            map.insert(
                "strcat",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let i8_ty = IntegerType::get(ctx, 8, Signedness::Signless);
                    let ptr_ty = PointerType::get(ctx, 0);
                    let llvm_func_ty =
                        FuncType::get(ctx, i8_ty.into(), vec![ptr_ty.into(), ptr_ty.into()], false);
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            map.insert(
                "strcpy",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let i8_ty = IntegerType::get(ctx, 8, Signedness::Signless);
                    let ptr_ty = PointerType::get(ctx, 0);
                    let llvm_func_ty =
                        FuncType::get(ctx, i8_ty.into(), vec![ptr_ty.into(), ptr_ty.into()], false);
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            map.insert(
                "strncpy",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let i8_ty = IntegerType::get(ctx, 8, Signedness::Signless);
                    let ptr_ty = PointerType::get(ctx, 0);
                    let llvm_func_ty =
                        FuncType::get(ctx, i8_ty.into(), vec![ptr_ty.into(), ptr_ty.into()], false);
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            // similar to realloc, but for strndup
            map.insert(
                "strndup",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let i8_ty = PointerType::get(ctx, 0);
                    let first_arg = PointerType::get(ctx, 0);
                    let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
                    let llvm_func_ty = FuncType::get(
                        ctx,
                        i8_ty.into(),
                        vec![first_arg.into(), i64_ty.into()],
                        false,
                    );
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            map.insert(
                "exit",
                Box::new(|ctx, rewriter, args, callee_ident| {
                    let void_ret = VoidType::get(ctx);
                    let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
                    let llvm_func_ty =
                        FuncType::get(ctx, void_ret.into(), vec![i32_ty.into()], false);
                    let llvm_call = LlvmCallOp::new(
                        ctx,
                        CallOpCallable::Direct(callee_ident),
                        llvm_func_ty,
                        args,
                    );
                    let call_res = llvm_call.get_result(ctx);
                    rewriter.insert_op(ctx, &llvm_call);
                    let sext = SExtOp::new(ctx, call_res, res_ty);
                    let final_res = sext.get_result(ctx);
                    rewriter.insert_op(ctx, &sext);
                    rewriter.replace_operation_with_values(
                        ctx,
                        op.get_operation(),
                        vec![final_res],
                    );
                    return Ok(());
                }),
            );
            map
        }
    }*/
}
