//! Translate from Kisumu_lang dialect IR to LLVM dialect IR.
//!
//! The entry point is [`lower_module`], which applies the
//! [`DialectConversion`] infrastructure to convert a [`ModuleOp`] containing
//! Kisumu_lang dialect ops into one containing only LLVM dialect ops.
//!
//! # Design
//! Each Kisumu_lang op implements the [`ToLLVMDialect`] op-interface.
//! A [`KalToLLVM`] conversion driver matches all ops that implement this
//! interface and delegates each rewrite to the op itself.
//!
//! * `ConstantOp` -> `llvm.constant`
//! * `DeclOp`     -> `llvm.alloca`
//! * `LoadOp`     -> `llvm.load`
//! * `StoreOp`    -> `llvm.store`
//! * `BinOp`      -> `llvm.add`/`sub`/`mul` or `llvm.icmp` + `llvm.sext`
//! * `CallOp`     -> `llvm.call`
//! * `ReturnOp`   -> `llvm.return`
//! * `YieldOp`    -> erased (handled by parent IfOp / WhileOp)
//! * `IfOp`       -> CFG: then / else / merge blocks
//! * `WhileOp`    -> CFG: header / body / exit blocks

use crate::dialect::{
    BinOp, BinOpKind, CallOp as KalCallOp, ConstantOp as KalConstantOp, DeclOp as KalDeclOp,
    IfOp as KalIfOp, LoadOp as KalLoadOp, ReturnOp as KalReturnOp, StoreOp as KalStoreOp,
    StringOp as KalStringOp, WhileOp as KalWhileOp, YieldOp as KalYieldOp,
};
use awint::bw;
use pliron::builtin::op_interfaces::SingleBlockRegionInterface;
use pliron::identifier::Identifier;
use pliron::{
    builtin::{
        self,
        attributes::IntegerAttr,
        op_interfaces::{
            CallOpCallable, OneRegionInterface, OneResultInterface, SymbolOpInterface,
        },
        ops::{ConstantOp as BuiltinConstantOp, ModuleOp},
        types::{FunctionType, IntegerType, Signedness},
    },
    context::{Context, Ptr},
    derive::op_interface_impl,
    irbuild::{
        IRStatus,
        dialect_conversion::{
            DialectConversion, DialectConversionRewriter, OperandsInfo, apply_dialect_conversion,
        },
        inserter::{BlockInsertionPoint, Inserter, OpInsertionPoint},
        rewriter::Rewriter,
    },
    linked_list::ContainsLinkedList,
    op::{Op, op_cast, op_impls},
    operation::Operation,
    result::Result,
    r#type::{TypeHandle, Typed},
    utils::apint::APInt,
    value::Value,
};
use pliron_llvm::{
    ToLLVMDialect,
    attributes::LinkageAttr,
    ops::{AddressOfOp, GlobalOp, InsertValueOp, ZeroOp},
    types::{ArrayType, PointerType},
};
use pliron_llvm::{
    attributes::{ICmpPredicateAttr, IntegerOverflowFlagsAttr},
    op_interfaces::{BinArithOp, CastOpInterface, IntBinArithOpWithOverflowFlag},
    ops::{
        AddOp, AllocaOp, AndOp, BrOp, CallOp as LlvmCallOp, CondBrOp, ICmpOp, LoadOp as LlvmLoadOp,
        MulOp, OrOp, ReturnOp as LlvmReturnOp, SDivOp, SExtOp, SRemOp, StoreOp as LlvmStoreOp,
        SubOp, XorOp,
    },
    types::FuncType,
};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static STRING_GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

// ─── DialectConversion driver ───────────────────────────────────────────────

/// Conversion driver: matches any Kisumu_lang op that implements
/// [`ToLLVMDialect`] and delegates the rewrite to the op itself.
// ANCHOR: kal_to_llvm_driver
pub struct KalToLLVM;

impl DialectConversion for KalToLLVM {
    fn can_convert_op(&self, ctx: &Context, op: Ptr<Operation>) -> bool {
        op_impls::<dyn ToLLVMDialect>(&*Operation::get_op_dyn(op, ctx))
            || Operation::get_op::<builtin::ops::FuncOp>(op, ctx).is_some()
    }

    fn rewrite(
        &mut self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        op: Ptr<Operation>,
        operands_info: &OperandsInfo,
    ) -> Result<()> {
        if let Some(func_op) = Operation::get_op::<builtin::ops::FuncOp>(op, ctx) {
            // Convert from builtin.func to llvm.func by updating the function type and argument types.
            return lower_func_op_to_llvm(&func_op, ctx, rewriter);
        }
        let op_dyn = Operation::get_op_dyn(op, ctx);
        let to_llvm_op = op_cast::<dyn ToLLVMDialect>(&*op_dyn)
            .expect("Matched Op must implement ToLLVMDialect");
        to_llvm_op.rewrite(ctx, rewriter, operands_info)
    }
}
// ANCHOR_END: kal_to_llvm_driver

// ─── Public API ─────────────────────────────────────────────────────────────

// ANCHOR: lower_module
/// Lower a [`ModuleOp`] containing Kisumu_lang dialect ops in place.
///
/// Uses the [`DialectConversion`] infrastructure: each Kisumu_lang op
/// implements [`ToLLVMDialect`] and knows how to lower itself to LLVM ops.
pub fn lower_module(ctx: &mut Context, module: ModuleOp) -> Result<IRStatus> {
    apply_dialect_conversion(ctx, &mut KalToLLVM, module.get_operation())
}
// ANCHOR_END: lower_module

// ─── ToLLVMDialect implementations ─────────────────────────────────────────

// ── kisumu_lang.constant -> llvm.constant ───────────────────────────────────
// ANCHOR: constant_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalConstantOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let val_attr = self.value_attr(ctx);
        let llvm_const = BuiltinConstantOp::new(ctx, Box::new(val_attr));
        let new_result = llvm_const.get_result(ctx);
        rewriter.insert_op(ctx, &llvm_const);
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![new_result]);
        Ok(())
    }
}
// ANCHOR_END: constant_to_llvm

fn create_string_global_and_address(ctx: &mut Context, s: &str) -> Result<(GlobalOp, AddressOfOp)> {
    // Null-terminated bytes (MLIR/LLVM string globals do *not* add `\0` for you).
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    let len = bytes.len() as u64;

    // Type: [len x i8]
    let i8_ty = IntegerType::get(ctx, 8, Signedness::Signless);
    let array_ty: TypeHandle = ArrayType::get(ctx, i8_ty.into(), len).into();

    // Unique symbol name: `.str.0`, `.str.1`, …
    let id = STRING_GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name: Identifier = format!("str_{id}")
        .as_str()
        .try_into()
        .expect("valid identifier");

    // llvm.global
    let global = GlobalOp::new(ctx, name.clone(), array_ty);
    global.set_attr_llvm_global_linkage(ctx, LinkageAttr::PrivateLinkage);

    global.add_initializer_region(ctx);
    let init_block = global.get_initializer_block(ctx).unwrap();
    let mut ins = pliron::irbuild::inserter::IRInserter::<pliron::irbuild::listener::DummyListener>::new_at_block_end(init_block);

    let zero = ZeroOp::new(ctx, array_ty);
    let mut curr_val = zero.get_result(ctx);
    ins.append_op(ctx, &zero);

    for (i, &byte) in bytes.iter().enumerate() {
        let byte_attr = IntegerAttr::new(i8_ty, APInt::from_u64(byte as u64, bw(8)));
        let byte_const = BuiltinConstantOp::new(ctx, Box::new(byte_attr));
        let byte_val = byte_const.get_result(ctx);
        ins.append_op(ctx, &byte_const);

        let insert = InsertValueOp::new(ctx, curr_val, byte_val, vec![i as u32]);
        curr_val = insert.get_result(ctx);
        ins.append_op(ctx, &insert);
    }

    let ret = LlvmReturnOp::new(ctx, Some(curr_val));
    ins.append_op(ctx, &ret);

    // llvm.addressof → !llvm.ptr (addrspace 0)
    let addr = AddressOfOp::new(ctx, name, /*address_space=*/ 0);

    Ok((global, addr))
}

/// Insert the global into the **module** (not the current function block).
fn insert_global_into_module(
    ctx: &mut Context,
    rewriter: &mut pliron::irbuild::dialect_conversion::DialectConversionRewriter,
    string_op: &KalStringOp,
    global: &GlobalOp,
) {
    // Walk up to the ModuleOp that owns this function / op.
    let mut cur = string_op.get_operation();
    loop {
        let parent = cur.deref(ctx).get_parent_op(ctx);
        match parent {
            Some(p) if Operation::get_op::<ModuleOp>(p, ctx).is_some() => {
                // Append the global as a top-level op in the module.
                // ModuleOp::append_operation is the usual API in the kaleidoscope example.
                if let Some(module) = Operation::get_op::<ModuleOp>(p, ctx) {
                    module.append_operation(ctx, global.get_operation(), 0);
                }
                return;
            }
            Some(p) => cur = p,
            None => {
                // Fallback: insert via the rewriter at the current point
                // (still works if the conversion driver runs at module scope).
                rewriter.insert_op(ctx, global);
                return;
            }
        }
    }
}

#[op_interface_impl]
impl ToLLVMDialect for KalStringOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut pliron::irbuild::dialect_conversion::DialectConversionRewriter,
        _operands_info: &pliron::irbuild::dialect_conversion::OperandsInfo,
    ) -> Result<()> {
        // Extract the StringAttr stored on the Kisumu_lang string op.
        let str_attr = self.value_attr(ctx); // -> StringAttr
        let s = str_attr.clone();
        let s_owned: String = { s.into() };

        let (global, addr) = create_string_global_and_address(ctx, &s_owned)?;

        // Global lives at module scope.
        insert_global_into_module(ctx, rewriter, self, &global);

        // Address is an SSA value in the current block.
        let ptr: Value = addr.get_result(ctx);
        rewriter.insert_op(ctx, &addr);
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![ptr]);

        Ok(())
    }
}

// ── kisumu_lang.decl -> llvm.alloca ─────────────────────────────────────────
// ANCHOR: decl_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalDeclOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
        let elem_ty = self.variable_type(ctx);
        let size_attr = IntegerAttr::new(i32_ty, APInt::from_i32(1, bw(32)));
        let size_const = BuiltinConstantOp::new(ctx, Box::new(size_attr));
        let size_val = size_const.get_result(ctx);
        rewriter.insert_op(ctx, &size_const);
        let alloca = AllocaOp::new(ctx, elem_ty, size_val);
        let alloca_ptr = alloca.get_result(ctx);
        rewriter.insert_op(ctx, &alloca);
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![alloca_ptr]);
        Ok(())
    }
}
// ANCHOR_END: decl_to_llvm

// ── kisumu_lang.load -> llvm.load ───────────────────────────────────────────
// ANCHOR: load_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalLoadOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        operands_info: &OperandsInfo,
    ) -> Result<()> {
        // `operands_info` carries the already-converted slot operand.
        let slot = operands_info
            .lookup_most_recent_type(self.slot(ctx))
            .map_or(self.slot(ctx), |_| self.slot(ctx));
        let res_ty = self.get_operation().deref(ctx).get_result(0).get_type(ctx);
        let load = LlvmLoadOp::new(ctx, slot, res_ty);
        let result = load.get_result(ctx);
        rewriter.insert_op(ctx, &load);
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![result]);
        Ok(())
    }
}
// ANCHOR_END: load_to_llvm

// ── kisumu_lang.store -> llvm.store ─────────────────────────────────────────
// ANCHOR: store_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalStoreOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let slot = self.slot(ctx);
        let value = self.stored_value(ctx);
        let store = LlvmStoreOp::new(ctx, value, slot);
        rewriter.insert_op(ctx, &store);
        rewriter.erase_operation(ctx, self.get_operation());
        Ok(())
    }
}
// ANCHOR_END: store_to_llvm

// ── kisumu_lang.binop -> llvm arithmetic / comparison ───────────────────────
// ANCHOR: binop_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for BinOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let lhs = self.lhs(ctx);
        let rhs = self.rhs(ctx);
        let kind = self.kind(ctx);

        let result = match kind {
            BinOpKind::Add => {
                let op = AddOp::new_with_overflow_flag(
                    ctx,
                    lhs,
                    rhs,
                    IntegerOverflowFlagsAttr::default(),
                );
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::Sub => {
                let op = SubOp::new_with_overflow_flag(
                    ctx,
                    lhs,
                    rhs,
                    IntegerOverflowFlagsAttr::default(),
                );
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::Mul => {
                let op = MulOp::new_with_overflow_flag(
                    ctx,
                    lhs,
                    rhs,
                    IntegerOverflowFlagsAttr::default(),
                );
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }

            // Impl shim
            BinOpKind::Mod => {
                let op = SRemOp::new(ctx, lhs, rhs);
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::Div => {
                let op = SDivOp::new(ctx, lhs, rhs);
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::BitwiseAnd => {
                let op = AndOp::new(ctx, lhs, rhs);
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::BitwiseOr => {
                let op = OrOp::new(ctx, lhs, rhs);
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            BinOpKind::BitwiseXor => {
                let op = XorOp::new(ctx, lhs, rhs);
                let r = op.get_result(ctx);
                rewriter.insert_op(ctx, &op);
                r
            }
            _ => {
                // Comparison: ICmpOp yields i1; sign-extend to i64.
                let pred = binop_kind_to_icmp_pred(kind);
                let icmp = ICmpOp::new(ctx, pred, lhs, rhs);
                let cmp_i1 = icmp.get_result(ctx);
                rewriter.insert_op(ctx, &icmp);
                let sext = SExtOp::new(ctx, cmp_i1, i64_ty.into());
                let r = sext.get_result(ctx);
                rewriter.insert_op(ctx, &sext);
                r
            }
        };
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![result]);
        Ok(())
    }
}
// ANCHOR_END: binop_to_llvm

// ── kisumu_lang.call -> llvm.call ───────────────────────────────────────────
// ANCHOR: call_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalCallOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let callee_attr = self
            .get_attr_callee(ctx)
            .expect("CallOp must have callee attribute")
            .clone();
        let callee_ident = pliron::identifier::Identifier::from(callee_attr);
        let n_args = self.get_operation().deref(ctx).get_num_operands();
        let args: Vec<Value> = (0..n_args)
            .map(|i| self.get_operation().deref(ctx).get_operand(i))
            .collect();
        let res_ty = self.get_operation().deref(ctx).get_result(0).get_type(ctx);

        if callee_ident.as_str() == "printf" {
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
            rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![final_res]);
            return Ok(());
        }

        if callee_ident.as_str() == "malloc" {
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
            rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![final_res]);
            return Ok(());
        }

        if callee_ident.as_str() == "strcat" || callee_ident.as_str() == "strcpy" {
            let i8_ty = PointerType::get(ctx, 0);
            let first_arg = PointerType::get(ctx, 0);
            // let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
            let llvm_func_ty = FuncType::get(
                ctx,
                i8_ty.into(),
                vec![first_arg.clone().into(), first_arg.into()],
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
            rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![final_res]);
            return Ok(());
        }

        if callee_ident.as_str() == "realloc" {
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
            rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![final_res]);
            return Ok(());
        }

        if callee_ident.as_str() == "strncpy" {
            let i8_ty = PointerType::get(ctx, 0);
            let first_arg = PointerType::get(ctx, 0);
            let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
            let llvm_func_ty = FuncType::get(
                ctx,
                i8_ty.into(),
                vec![first_arg.clone().into(), first_arg.into(), i64_ty.into()],
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
            rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![final_res]);
            return Ok(());
        }

        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let arg_types: Vec<TypeHandle> = args.iter().map(|arg| arg.get_type(ctx)).collect();
        let llvm_func_ty = FuncType::get(ctx, i64_ty.into(), arg_types, false);
        let llvm_call = LlvmCallOp::new(
            ctx,
            CallOpCallable::Direct(callee_ident),
            llvm_func_ty,
            args,
        );
        let result = llvm_call.get_result(ctx);
        rewriter.insert_op(ctx, &llvm_call);
        rewriter.replace_operation_with_values(ctx, self.get_operation(), vec![result]);
        Ok(())
    }
}
// ANCHOR_END: call_to_llvm

// ── kisumu_lang.return -> llvm.return ───────────────────────────────────────
// ANCHOR: return_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalReturnOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let val = self.value(ctx);
        let ret = LlvmReturnOp::new(ctx, Some(val));
        rewriter.insert_op(ctx, &ret);
        rewriter.erase_operation(ctx, self.get_operation());
        Ok(())
    }
}
// ANCHOR_END: return_to_llvm

// ── kisumu_lang.yield -> erase ──────────────────────────────────────────────
// YieldOp is an IsTerminatorInterface impl, so it must be handled.
// It's handled by the parent IfOp/WhileOp's rewrite, but may also be
// matched here; in that case just erase it.
#[op_interface_impl]
impl ToLLVMDialect for KalYieldOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        rewriter.erase_operation(ctx, self.get_operation());
        Ok(())
    }
}

// ── kisumu_lang.if -> CFG: then / else / merge blocks ───────────────────────
// ANCHOR: if_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalIfOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let cond = self.condition(ctx);
        let then_region = self.then_region(ctx);
        let else_region = self.else_region(ctx);

        // The entry block of each region is the single block in the region.
        let then_entry = then_region
            .deref(ctx)
            .get_head()
            .expect("IfOp then_region must have a block");
        let else_entry = else_region
            .deref(ctx)
            .get_head()
            .expect("IfOp else_region must have a block");

        let then_term = then_entry
            .deref(ctx)
            .get_terminator(ctx)
            .expect("then block must have a terminator");
        let else_term = else_entry
            .deref(ctx)
            .get_terminator(ctx)
            .expect("else block must have a terminator");

        // Convert the i64 condition to i1 via `icmp ne cond, 0`.
        let zero_attr = IntegerAttr::new(i64_ty, APInt::from_i64(0, bw(64)));
        let zero_const = BuiltinConstantOp::new(ctx, Box::new(zero_attr));
        let zero_val = zero_const.get_result(ctx);
        rewriter.insert_op(ctx, &zero_const);
        let cmp = ICmpOp::new(ctx, ICmpPredicateAttr::NE, cond, zero_val);
        let cmp_i1 = cmp.get_result(ctx);
        rewriter.insert_op(ctx, &cmp);

        // Split the current block at the IfOp position to create the merge block.
        let pre_if_block = self
            .get_operation()
            .deref(ctx)
            .get_parent_block()
            .expect("IfOp must be in a block");
        let merge_block = rewriter.split_block(
            ctx,
            pre_if_block,
            OpInsertionPoint::BeforeOperation(self.get_operation()),
            Some("if_merge".try_into().unwrap()),
        );

        // Emit conditional branch in pre_if_block.
        rewriter.set_insertion_point(OpInsertionPoint::AtBlockEnd(pre_if_block));
        let cond_br = CondBrOp::new(ctx, cmp_i1, then_entry, vec![], else_entry, vec![]);
        rewriter.insert_op(ctx, &cond_br);

        // Replace YieldOp in then-branch with branch to merge.
        if Operation::is_op::<KalYieldOp>(then_term, ctx) {
            rewriter.set_insertion_point(OpInsertionPoint::BeforeOperation(then_term));
            let then_br = BrOp::new(ctx, merge_block, vec![]);
            rewriter.insert_op(ctx, &then_br);
            rewriter.erase_operation(ctx, then_term);
        }

        // Replace YieldOp in else-branch with branch to merge.
        if Operation::is_op::<KalYieldOp>(else_term, ctx) {
            rewriter.set_insertion_point(OpInsertionPoint::BeforeOperation(else_term));
            let else_br = BrOp::new(ctx, merge_block, vec![]);
            rewriter.insert_op(ctx, &else_br);
            rewriter.erase_operation(ctx, else_term);
        }

        // Inline both regions after the pre_if_block.
        rewriter.inline_region(
            ctx,
            then_region,
            BlockInsertionPoint::AfterBlock(pre_if_block),
        );
        rewriter.inline_region(
            ctx,
            else_region,
            BlockInsertionPoint::AfterBlock(then_entry),
        );

        // The IfOp itself has no results, so just erase it.
        rewriter.erase_operation(ctx, self.get_operation());
        Ok(())
    }
}
// ANCHOR_END: if_to_llvm

// ── kisumu_lang.while -> CFG: header / body / exit blocks ───────────────────
// ANCHOR: while_to_llvm
#[op_interface_impl]
impl ToLLVMDialect for KalWhileOp {
    fn rewrite(
        &self,
        ctx: &mut Context,
        rewriter: &mut DialectConversionRewriter,
        _operands_info: &OperandsInfo,
    ) -> Result<()> {
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let cond_ptr = self.cond_ptr(ctx);
        let body_region = self.body_region(ctx);
        let body_entry = body_region
            .deref(ctx)
            .get_head()
            .expect("WhileOp body_region must have a block");

        // Erase the YieldOp at the end of the body.
        let while_term = body_entry
            .deref(ctx)
            .get_terminator(ctx)
            .expect("body block must have a terminator");

        // The block containing the WhileOp is split to create the exit block.
        let pre_while_block = self
            .get_operation()
            .deref(ctx)
            .get_parent_block()
            .expect("WhileOp must be in a block");
        let exit_block = rewriter.split_block(
            ctx,
            pre_while_block,
            OpInsertionPoint::BeforeOperation(self.get_operation()),
            Some("while_exit".try_into().unwrap()),
        );

        // Create the header block.
        let header_block = rewriter.create_block(
            ctx,
            BlockInsertionPoint::AfterBlock(pre_while_block),
            Some("while_header".try_into().unwrap()),
            vec![],
        );

        // Emit an unconditional branch into the header from pre_while_block.
        rewriter.set_insertion_point(OpInsertionPoint::AtBlockEnd(pre_while_block));
        let br_to_header = BrOp::new(ctx, header_block, vec![]);
        rewriter.insert_op(ctx, &br_to_header);

        // Header: load condition, compare to zero, cond_br to body or exit.
        rewriter.set_insertion_point(OpInsertionPoint::AtBlockEnd(header_block));
        let cond_load = LlvmLoadOp::new(ctx, cond_ptr, i64_ty.into());
        let cond_i64 = cond_load.get_result(ctx);
        rewriter.insert_op(ctx, &cond_load);
        let zero_attr = IntegerAttr::new(i64_ty, APInt::from_i64(0, bw(64)));
        let zero_const = BuiltinConstantOp::new(ctx, Box::new(zero_attr));
        let zero_val = zero_const.get_result(ctx);
        rewriter.insert_op(ctx, &zero_const);
        let cmp = ICmpOp::new(ctx, ICmpPredicateAttr::NE, cond_i64, zero_val);
        let cmp_i1 = cmp.get_result(ctx);
        rewriter.insert_op(ctx, &cmp);
        let cond_br = CondBrOp::new(ctx, cmp_i1, body_entry, vec![], exit_block, vec![]);
        rewriter.insert_op(ctx, &cond_br);

        // Replace YieldOp at end of body with back-edge to header.
        if Operation::is_op::<KalYieldOp>(while_term, ctx) {
            rewriter.set_insertion_point(OpInsertionPoint::BeforeOperation(while_term));
            let back_edge = BrOp::new(ctx, header_block, vec![]);
            rewriter.insert_op(ctx, &back_edge);
            rewriter.erase_operation(ctx, while_term);
        }

        // Inline the body region after the header block.
        rewriter.inline_region(
            ctx,
            body_region,
            BlockInsertionPoint::AfterBlock(header_block),
        );

        // Erase the WhileOp.
        rewriter.erase_operation(ctx, self.get_operation());
        Ok(())
    }
}
// ANCHOR_END: while_to_llvm

// Convert from builtin.func to llvm.func by updating the function type and argument types.
fn lower_func_op_to_llvm(
    func_op: &builtin::ops::FuncOp,
    ctx: &mut Context,
    rewriter: &mut DialectConversionRewriter,
) -> Result<()> {
    let func_name = func_op.get_symbol_name(ctx);
    if func_name.as_str() == "printf" {
        let i32_ty = IntegerType::get(ctx, 32, Signedness::Signless);
        let ptr_ty = PointerType::get(ctx, 0);
        let llvm_func_ty = FuncType::get(ctx, i32_ty.into(), vec![ptr_ty.into()], true);
        let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
        let llvm_func_op_ptr = llvm_func_op.get_operation();
        rewriter.insert_op(ctx, &llvm_func_op);
        rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
        return Ok(());
    }

    if func_name.as_str() == "malloc" {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let llvm_func_ty = FuncType::get(ctx, i8_ptr_ty.into(), vec![i64_ty.into()], false);
        let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
        let llvm_func_op_ptr = llvm_func_op.get_operation();
        rewriter.insert_op(ctx, &llvm_func_op);
        rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
        return Ok(());
    }

    if func_name.as_str() == "realloc" {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let first_arg = i8_ptr_ty.clone();
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let llvm_func_ty = FuncType::get(
            ctx,
            i8_ptr_ty.into(),
            vec![first_arg.into(), i64_ty.into()],
            false,
        );
        let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
        let llvm_func_op_ptr = llvm_func_op.get_operation();
        rewriter.insert_op(ctx, &llvm_func_op);
        rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
        return Ok(());
    }

    if func_name.as_str() == "strcat" || func_name.as_str() == "strcpy" {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let first_arg = i8_ptr_ty.clone();
        // let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let llvm_func_ty = FuncType::get(
            ctx,
            i8_ptr_ty.into(),
            vec![first_arg.clone().into(), first_arg.into()],
            false,
        );
        let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
        let llvm_func_op_ptr = llvm_func_op.get_operation();
        rewriter.insert_op(ctx, &llvm_func_op);
        rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
        return Ok(());
    }

    if func_name.as_str() == "strncpy" {
        let i8_ptr_ty = PointerType::get(ctx, 0);
        let first_arg = i8_ptr_ty.clone();
        let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
        let llvm_func_ty = FuncType::get(
            ctx,
            i8_ptr_ty.into(),
            vec![first_arg.clone().into(), first_arg.into(), i64_ty.into()],
            false,
        );
        let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
        let llvm_func_op_ptr = llvm_func_op.get_operation();
        rewriter.insert_op(ctx, &llvm_func_op);
        rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
        return Ok(());
    }

    let i64_ty = IntegerType::get(ctx, 64, Signedness::Signless);
    let func_entry = func_op.get_entry_block(ctx);

    // All args are i64, and the return type is i64.
    let n_args = func_op.get_entry_block(ctx).deref(ctx).get_num_arguments();
    let arg_types: Vec<TypeHandle> = (0..n_args).map(|_| i64_ty.into()).collect();
    let llvm_func_ty = FuncType::get(ctx, i64_ty.into(), arg_types, false);
    let llvm_func_op = pliron_llvm::ops::FuncOp::new(ctx, func_name, llvm_func_ty);
    let llvm_func_op_ptr = llvm_func_op.get_operation();
    let llvm_entry = llvm_func_op.get_or_create_entry_block(ctx);
    rewriter.insert_op(ctx, &llvm_func_op);

    // Move the region from the original func_op to the new llvm.func op.
    rewriter.inline_region(
        ctx,
        func_op.get_region(ctx),
        BlockInsertionPoint::AfterBlock(llvm_entry),
    );

    // Branch from the new entry block to the original entry block
    // (now inlined after the new entry block).
    let args: Vec<_> = llvm_entry.deref(ctx).arguments().collect();
    let br = BrOp::new(ctx, func_entry, args);
    rewriter.set_insertion_point(OpInsertionPoint::AtBlockEnd(llvm_entry));
    rewriter.insert_op(ctx, &br);

    // Replace the original FuncOp with the new llvm.func op.
    rewriter.replace_operation(ctx, func_op.get_operation(), llvm_func_op_ptr);
    Ok(())
}

// ─── Helper ────────────────────────────────────────────────────────────────

// The method used in declaring this printf function is to be followed in order
// to eventually do away with the hack that involves copying libc functions
// for final linking by clang.
//
// This has the potential to eliminate the dependence on clang.
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
    let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
    module.append_operation(ctx, malloc_decl.get_operation(), 0);
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
    let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
    module.append_operation(ctx, malloc_decl.get_operation(), 0);
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
    let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
    module.append_operation(ctx, malloc_decl.get_operation(), 0);
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
    let malloc_decl = pliron::builtin::ops::FuncOp::new(ctx, name, func_ty);
    module.append_operation(ctx, malloc_decl.get_operation(), 0);
}

/// Map a comparison [`BinOpKind`] to the corresponding [`ICmpPredicateAttr`].
fn binop_kind_to_icmp_pred(kind: BinOpKind) -> ICmpPredicateAttr {
    match kind {
        BinOpKind::Lt => ICmpPredicateAttr::SLT,
        BinOpKind::Gt => ICmpPredicateAttr::SGT,
        BinOpKind::Le => ICmpPredicateAttr::SLE,
        BinOpKind::Ge => ICmpPredicateAttr::SGE,
        BinOpKind::Eq => ICmpPredicateAttr::EQ,
        BinOpKind::Ne => ICmpPredicateAttr::NE,
        _ => panic!("binop_kind_to_icmp_pred: not a comparison op {}", kind),
    }
}
