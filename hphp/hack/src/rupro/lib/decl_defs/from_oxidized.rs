// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use crate::decl_defs::{self, folded, shallow, ty, DeclTy, DeclTy_};
use crate::reason::Reason;
use pos::Pos;

use oxidized_by_ref as obr;

#[inline]
fn slice<T: Copy + Into<U>, U>(items: &[T]) -> Box<[U]> {
    items.iter().copied().map(Into::into).collect()
}

#[inline]
fn map<'a, K1, V1, K2, V2, M>(items: impl Iterator<Item = (&'a K1, &'a V1)>) -> M
where
    K1: Copy + Into<K2> + 'a,
    V1: Copy + Into<V2> + 'a,
    M: FromIterator<(K2, V2)>,
{
    items.map(|(&k, &v)| (k.into(), v.into())).collect()
}

impl From<obr::ast_defs::XhpEnumValue<'_>> for ty::XhpEnumValue {
    fn from(x: obr::ast_defs::XhpEnumValue<'_>) -> Self {
        use obr::ast_defs::XhpEnumValue as Obr;
        match x {
            Obr::XEVInt(i) => Self::XEVInt(i),
            Obr::XEVString(s) => Self::XEVString(s.into()),
        }
    }
}

impl From<obr::typing_defs::CeVisibility<'_>> for ty::CeVisibility {
    fn from(x: obr::typing_defs::CeVisibility<'_>) -> Self {
        use obr::typing_defs::CeVisibility as Obr;
        match x {
            Obr::Vpublic => Self::Public,
            Obr::Vprivate(s) => Self::Private(s.into()),
            Obr::Vprotected(s) => Self::Protected(s.into()),
            Obr::Vinternal(s) => Self::Internal(s.into()),
        }
    }
}

impl From<obr::typing_defs::IfcFunDecl<'_>> for ty::IfcFunDecl {
    fn from(x: obr::typing_defs::IfcFunDecl<'_>) -> Self {
        use obr::typing_defs_core::IfcFunDecl as Obr;
        match x {
            Obr::FDPolicied(s) => Self::FDPolicied(s.map(Into::into)),
            Obr::FDInferFlows => Self::FDInferFlows,
        }
    }
}

fn tshape_field_name_from_decl<P: Pos>(
    x: obr::typing_defs::TshapeFieldName<'_>,
) -> (ty::ShapeFieldNamePos<P>, ty::TshapeFieldName) {
    use obr::typing_defs_core::TshapeFieldName as Obr;
    use ty::ShapeFieldNamePos as SfnPos;
    use ty::TshapeFieldName;
    match x {
        Obr::TSFlitInt(&pos_id) => (
            SfnPos::Simple(pos_id.0.into()),
            TshapeFieldName::TSFlitInt(pos_id.1.into()),
        ),
        Obr::TSFlitStr(&pos_bytes) => (
            SfnPos::Simple(pos_bytes.0.into()),
            TshapeFieldName::TSFlitStr(pos_bytes.1.into()),
        ),
        Obr::TSFclassConst(&(pos_id1, pos_id2)) => (
            SfnPos::ClassConst(pos_id1.0.into(), pos_id2.0.into()),
            TshapeFieldName::TSFclassConst(pos_id1.1.into(), pos_id2.1.into()),
        ),
    }
}

impl<P: Pos> From<&obr::typing_defs::UserAttribute<'_>> for ty::UserAttribute<P> {
    fn from(attr: &obr::typing_defs::UserAttribute<'_>) -> Self {
        Self {
            name: attr.name.into(),
            classname_params: (attr.classname_params.iter())
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::Tparam<'_>> for ty::Tparam<R, DeclTy<R>> {
    fn from(tparam: &obr::typing_defs::Tparam<'_>) -> Self {
        Self {
            variance: tparam.variance,
            name: tparam.name.into(),
            tparams: slice(tparam.tparams),
            constraints: (tparam.constraints.iter())
                .map(|(kind, ty)| (*kind, (*ty).into()))
                .collect(),
            reified: tparam.reified,
            user_attributes: slice(tparam.user_attributes),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::WhereConstraint<'_>> for ty::WhereConstraint<DeclTy<R>> {
    fn from(x: &obr::typing_defs::WhereConstraint<'_>) -> Self {
        Self(x.0.into(), x.1, x.2.into())
    }
}

fn decl_shape_field_type<R: Reason>(
    field_name_pos: ty::ShapeFieldNamePos<R::Pos>,
    sft: &obr::typing_defs::ShapeFieldType<'_>,
) -> ty::ShapeFieldType<R> {
    ty::ShapeFieldType {
        field_name_pos,
        optional: sft.optional,
        ty: sft.ty.into(),
    }
}

impl<R: Reason> From<&obr::typing_defs::Ty<'_>> for DeclTy<R> {
    fn from(ty: &obr::typing_defs::Ty<'_>) -> Self {
        use obr::typing_defs_core::Ty_::*;
        use DeclTy_::*;
        let reason = R::from(*ty.0);
        let ty_ = match ty.1 {
            Tthis => DTthis,
            Tapply(&(pos_id, tys)) => DTapply(Box::new((pos_id.into(), slice(tys)))),
            Tmixed => DTmixed,
            Tlike(ty) => DTlike(ty.into()),
            Tany(_) => DTany,
            Terr => DTerr,
            Tnonnull => DTnonnull,
            Tdynamic => DTdynamic,
            Toption(ty) => DToption(ty.into()),
            Tprim(prim) => DTprim(*prim),
            Tfun(ft) => DTfun(Box::new(ft.into())),
            Ttuple(tys) => DTtuple(slice(tys)),
            Tshape(&(kind, fields)) => DTshape(Box::new((
                kind,
                fields
                    .iter()
                    .map(|(name, ty)| {
                        let (field_name_pos, name) = tshape_field_name_from_decl(name.0);
                        (name, decl_shape_field_type(field_name_pos, ty))
                    })
                    .collect(),
            ))),
            Tvar(ident) => DTvar(ident.into()),
            Tgeneric(&(pos_id, tys)) => DTgeneric(Box::new((pos_id.into(), slice(tys)))),
            Tunion(tys) => DTunion(slice(tys)),
            Tintersection(tys) => DTintersection(slice(tys)),
            TvecOrDict(&(ty1, ty2)) => DTvecOrDict(Box::new((ty1.into(), ty2.into()))),
            Taccess(taccess_type) => DTaccess(Box::new(taccess_type.into())),
            TunappliedAlias(_) | Tnewtype(_) | Tdependent(_) | Tclass(_) | Tneg(_) => {
                unreachable!("Not used in decl tys")
            }
        };
        DeclTy::new(reason, ty_)
    }
}

impl<R: Reason> From<&obr::typing_defs::TaccessType<'_>> for ty::TaccessType<R, DeclTy<R>> {
    fn from(taccess_type: &obr::typing_defs::TaccessType<'_>) -> Self {
        Self {
            ty: taccess_type.0.into(),
            type_const: taccess_type.1.into(),
        }
    }
}

impl<R: Reason> From<obr::typing_defs::Capability<'_>> for ty::Capability<R, DeclTy<R>> {
    fn from(cap: obr::typing_defs::Capability<'_>) -> Self {
        use obr::typing_defs_core::Capability as Obr;
        match cap {
            Obr::CapDefaults(pos) => Self::CapDefaults(pos.into()),
            Obr::CapTy(ty) => Self::CapTy(ty.into()),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::FunImplicitParams<'_>>
    for ty::FunImplicitParams<R, DeclTy<R>>
{
    fn from(x: &obr::typing_defs::FunImplicitParams<'_>) -> Self {
        Self {
            capability: x.capability.into(),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::FunType<'_>> for ty::FunType<R, DeclTy<R>> {
    fn from(ft: &obr::typing_defs::FunType<'_>) -> Self {
        Self {
            tparams: slice(ft.tparams),
            where_constraints: slice(ft.where_constraints),
            params: slice(ft.params),
            implicit_params: ft.implicit_params.into(),
            ret: ft.ret.into(),
            flags: ft.flags,
            ifc_decl: ft.ifc_decl.into(),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs_core::PossiblyEnforcedTy<'_>>
    for decl_defs::ty::PossiblyEnforcedTy<DeclTy<R>>
{
    fn from(ty: &obr::typing_defs_core::PossiblyEnforcedTy<'_>) -> Self {
        Self {
            ty: ty.type_.into(),
            enforced: ty.enforced,
        }
    }
}

impl<R: Reason> From<&obr::typing_defs_core::FunParam<'_>> for ty::FunParam<R, DeclTy<R>> {
    fn from(fp: &obr::typing_defs_core::FunParam<'_>) -> Self {
        Self {
            pos: fp.pos.into(),
            name: fp.name.map(Into::into),
            ty: fp.type_.into(),
            flags: fp.flags,
        }
    }
}

impl From<obr::typing_defs::ClassConstFrom<'_>> for ty::ClassConstFrom {
    fn from(x: obr::typing_defs::ClassConstFrom<'_>) -> Self {
        use obr::typing_defs::ClassConstFrom as Obr;
        match x {
            Obr::Self_ => Self::Self_,
            Obr::From(s) => Self::From(s.into()),
        }
    }
}

impl From<obr::typing_defs::ClassConstRef<'_>> for ty::ClassConstRef {
    fn from(x: obr::typing_defs::ClassConstRef<'_>) -> Self {
        Self(x.0.into(), x.1.into())
    }
}

impl<R: Reason> From<&obr::typing_defs::AbstractTypeconst<'_>> for ty::AbstractTypeconst<R> {
    fn from(x: &obr::typing_defs::AbstractTypeconst<'_>) -> Self {
        Self {
            as_constraint: x.as_constraint.map(Into::into),
            super_constraint: x.super_constraint.map(Into::into),
            default: x.default.map(Into::into),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::ConcreteTypeconst<'_>> for ty::ConcreteTypeconst<R> {
    fn from(x: &obr::typing_defs::ConcreteTypeconst<'_>) -> Self {
        Self {
            ty: x.tc_type.into(),
        }
    }
}

impl<R: Reason> From<obr::typing_defs::Typeconst<'_>> for ty::Typeconst<R> {
    fn from(x: obr::typing_defs::Typeconst<'_>) -> Self {
        use obr::typing_defs::Typeconst as Obr;
        match x {
            Obr::TCAbstract(atc) => Self::TCAbstract(atc.into()),
            Obr::TCConcrete(ctc) => Self::TCConcrete(ctc.into()),
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::EnumType<'_>> for ty::EnumType<R> {
    fn from(x: &obr::typing_defs::EnumType<'_>) -> Self {
        Self {
            base: x.base.into(),
            constraint: x.constraint.map(Into::into),
            includes: slice(x.includes),
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ShallowClassConst<'_>>
    for shallow::ShallowClassConst<R>
{
    fn from(scc: &obr::shallow_decl_defs::ShallowClassConst<'_>) -> Self {
        Self {
            kind: scc.abstract_,
            name: scc.name.into(),
            ty: scc.type_.into(),
            refs: slice(scc.refs),
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ShallowTypeconst<'_>>
    for shallow::ShallowTypeconst<R>
{
    fn from(stc: &obr::shallow_decl_defs::ShallowTypeconst<'_>) -> Self {
        Self {
            name: stc.name.into(),
            kind: stc.kind.into(),
            enforceable: if stc.enforceable.1 {
                Some(stc.enforceable.0.into())
            } else {
                None
            },
            reifiable: stc.reifiable.map(Into::into),
            is_ctx: stc.is_ctx,
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ShallowMethod<'_>> for shallow::ShallowMethod<R> {
    fn from(sm: &obr::shallow_decl_defs::ShallowMethod<'_>) -> Self {
        Self {
            name: sm.name.into(),
            ty: sm.type_.into(),
            visibility: sm.visibility,
            deprecated: sm.deprecated.map(Into::into),
            attributes: slice(sm.attributes),
            flags: sm.flags,
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ShallowProp<'_>> for shallow::ShallowProp<R> {
    fn from(sp: &obr::shallow_decl_defs::ShallowProp<'_>) -> Self {
        Self {
            name: sp.name.into(),
            xhp_attr: sp.xhp_attr,
            ty: sp.type_.map(Into::into),
            visibility: sp.visibility,
            flags: sp.flags,
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ClassDecl<'_>> for shallow::ShallowClass<R> {
    fn from(sc: &obr::shallow_decl_defs::ClassDecl<'_>) -> Self {
        Self {
            mode: sc.mode,
            is_final: sc.final_,
            is_abstract: sc.abstract_,
            is_xhp: sc.is_xhp,
            has_xhp_keyword: sc.has_xhp_keyword,
            kind: sc.kind,
            module: sc.module.map(Into::into),
            name: sc.name.into(),
            tparams: slice(sc.tparams),
            where_constraints: slice(sc.where_constraints),
            extends: slice(sc.extends),
            uses: slice(sc.uses),
            xhp_attr_uses: slice(sc.xhp_attr_uses),
            xhp_enum_values: (sc.xhp_enum_values.iter())
                .map(|(&k, v)| (k.into(), slice(v)))
                .collect(),
            req_extends: slice(sc.req_extends),
            req_implements: slice(sc.req_implements),
            implements: slice(sc.implements),
            support_dynamic_type: sc.support_dynamic_type,
            consts: slice(sc.consts),
            typeconsts: slice(sc.typeconsts),
            props: slice(sc.props),
            static_props: slice(sc.sprops),
            constructor: sc.constructor.map(Into::into),
            static_methods: slice(sc.static_methods),
            methods: slice(sc.methods),
            user_attributes: slice(sc.user_attributes),
            enum_type: sc.enum_type.map(Into::into),
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::FunDecl<'_>> for shallow::FunDecl<R> {
    fn from(sf: &obr::shallow_decl_defs::FunDecl<'_>) -> Self {
        Self {
            pos: sf.pos.into(),
            ty: sf.type_.into(),
            deprecated: sf.deprecated.map(Into::into),
            module: sf.module.map(Into::into),
            internal: sf.internal,
            php_std_lib: sf.php_std_lib,
            support_dynamic_type: sf.support_dynamic_type,
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::TypedefDecl<'_>> for shallow::TypedefDecl<R> {
    fn from(x: &obr::shallow_decl_defs::TypedefDecl<'_>) -> Self {
        Self {
            module: x.module.map(Into::into),
            pos: x.pos.into(),
            vis: x.vis,
            tparams: slice(x.tparams),
            constraint: x.constraint.map(Into::into),
            ty: x.type_.into(),
            is_ctx: x.is_ctx,
            attributes: slice(x.attributes),
        }
    }
}

impl<R: Reason> From<&obr::shallow_decl_defs::ConstDecl<'_>> for shallow::ConstDecl<R> {
    fn from(x: &obr::shallow_decl_defs::ConstDecl<'_>) -> Self {
        Self {
            pos: x.pos.into(),
            ty: x.type_.into(),
        }
    }
}

impl<R: Reason> From<(&str, obr::shallow_decl_defs::Decl<'_>)> for shallow::Decl<R> {
    fn from(decl: (&str, obr::shallow_decl_defs::Decl<'_>)) -> Self {
        use obr::shallow_decl_defs::Decl as Obr;
        match decl {
            (name, Obr::Class(x)) => Self::Class(name.into(), x.into()),
            (name, Obr::Fun(x)) => Self::Fun(name.into(), x.into()),
            (name, Obr::Typedef(x)) => Self::Typedef(name.into(), x.into()),
            (name, Obr::Const(x)) => Self::Const(name.into(), x.into()),
        }
    }
}

impl From<&obr::decl_defs::Element<'_>> for folded::FoldedElement {
    fn from(x: &obr::decl_defs::Element<'_>) -> Self {
        Self {
            flags: x.flags,
            origin: x.origin.into(),
            visibility: x.visibility.into(),
            deprecated: x.deprecated.map(Into::into),
        }
    }
}

impl<R: Reason> From<&obr::decl_defs::SubstContext<'_>> for folded::SubstContext<R> {
    fn from(x: &obr::decl_defs::SubstContext<'_>) -> Self {
        Self {
            subst: folded::Subst(map(x.subst.iter())),
            class_context: x.class_context.into(),
            from_req_extends: x.from_req_extends,
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::TypeconstType<'_>> for folded::TypeConst<R> {
    fn from(x: &obr::typing_defs::TypeconstType<'_>) -> Self {
        Self {
            is_synthesized: x.synthesized,
            name: x.name.into(),
            kind: x.kind.into(),
            origin: x.origin.into(),
            enforceable: if x.enforceable.1 {
                Some(x.enforceable.0.into())
            } else {
                None
            },
            reifiable: x.reifiable.map(Into::into),
            is_concretized: x.concretized,
            is_ctx: x.is_ctx,
        }
    }
}

impl<R: Reason> From<&obr::typing_defs::ClassConst<'_>> for folded::ClassConst<R> {
    fn from(x: &obr::typing_defs::ClassConst<'_>) -> Self {
        Self {
            is_synthesized: x.synthesized,
            kind: x.abstract_,
            pos: x.pos.into(),
            ty: x.type_.into(),
            origin: x.origin.into(),
            refs: slice(x.refs),
        }
    }
}

impl<R: Reason> From<&obr::decl_defs::Requirement<'_>> for folded::Requirement<R> {
    fn from(req: &obr::decl_defs::Requirement<'_>) -> Self {
        Self(req.0.into(), req.1.into())
    }
}

impl<R: Reason> From<&obr::decl_defs::DeclClassType<'_>> for folded::FoldedClass<R> {
    fn from(cls: &obr::decl_defs::DeclClassType<'_>) -> Self {
        Self {
            name: cls.name.into(),
            pos: cls.pos.into(),
            kind: cls.kind,
            is_final: cls.final_,
            is_const: cls.const_,
            is_internal: cls.internal,
            is_xhp: cls.is_xhp,
            has_xhp_keyword: cls.has_xhp_keyword,
            support_dynamic_type: cls.support_dynamic_type,
            enum_type: cls.enum_type.map(Into::into),
            module: cls.module.map(Into::into),
            tparams: slice(cls.tparams),
            where_constraints: slice(cls.where_constraints),
            substs: map(cls.substs.iter()),
            ancestors: map(cls.ancestors.iter()),
            props: map(cls.props.iter()),
            static_props: map(cls.sprops.iter()),
            methods: map(cls.methods.iter()),
            static_methods: map(cls.smethods.iter()),
            constructor: cls.construct.0.map(Into::into), // TODO: ConsistentKind
            consts: map(cls.consts.iter()),
            type_consts: map(cls.typeconsts.iter()),
            xhp_enum_values: (cls.xhp_enum_values.iter())
                .map(|(&s, &evs)| (s.into(), slice(evs)))
                .collect(),
            extends: cls.extends.iter().copied().map(Into::into).collect(),
            xhp_attr_deps: cls.xhp_attr_deps.iter().copied().map(Into::into).collect(),
            req_ancestors: cls.req_ancestors.iter().copied().map(Into::into).collect(),
            req_ancestors_extends: (cls.req_ancestors_extends.iter())
                .copied()
                .map(Into::into)
                .collect(),
            sealed_whitelist: (cls.sealed_whitelist)
                .map(|l| l.iter().copied().map(Into::into).collect()),
            deferred_init_members: (cls.deferred_init_members.iter())
                .copied()
                .map(Into::into)
                .collect(),
            decl_errors: [].into(), // TODO
        }
    }
}