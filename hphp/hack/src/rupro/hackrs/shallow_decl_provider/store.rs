// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use super::TypeDecl;
use datastore::Store;
use pos::{ConstName, FunName, MethodName, ModuleName, PropName, TypeName};
use std::sync::Arc;
use ty::decl::{
    shallow::Decl, shallow::ModuleDecl, ConstDecl, FunDecl, ShallowClass, Ty, TypedefDecl,
};
use ty::reason::Reason;

/// A datastore for shallow declarations (i.e., the information we get from
/// decl-parsing a file). The backing datastores are permitted to evict their
/// contents at any time.
///
/// Consumers of a `ShallowDeclStore` expect the member-type accessors
/// (`get_method_type`, `get_constructor_type`, etc.) to be performant. For
/// instance, if our `Store` implementations store data in a serialized format,
/// looking up a method type should only deserialize that individual method, not
/// the entire `ShallowClass` containing that method declaration.
#[derive(Debug)]
pub struct ShallowDeclStore<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
    funs: Box<dyn Store<FunName, Arc<FunDecl<R>>>>,
    consts: Box<dyn Store<ConstName, Arc<ConstDecl<R>>>>,
    modules: Box<dyn Store<ModuleName, Arc<ModuleDecl<R>>>>,

    // The below tables are intended to be index tables for information stored
    // in the `types` table (the underlying data is shared via the `Hc` in
    // `Ty`). When inserting or removing from the `types` table, these
    // indices must be updated.
    properties: Box<dyn Store<(TypeName, PropName), Ty<R>>>,
    static_properties: Box<dyn Store<(TypeName, PropName), Ty<R>>>,
    methods: Box<dyn Store<(TypeName, MethodName), Ty<R>>>,
    static_methods: Box<dyn Store<(TypeName, MethodName), Ty<R>>>,
    constructors: Box<dyn Store<TypeName, Ty<R>>>,
}

impl<R: Reason> ShallowDeclStore<R> {
    pub fn new(
        types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
        funs: Box<dyn Store<FunName, Arc<FunDecl<R>>>>,
        consts: Box<dyn Store<ConstName, Arc<ConstDecl<R>>>>,
        modules: Box<dyn Store<ModuleName, Arc<ModuleDecl<R>>>>,
        properties: Box<dyn Store<(TypeName, PropName), Ty<R>>>,
        static_properties: Box<dyn Store<(TypeName, PropName), Ty<R>>>,
        methods: Box<dyn Store<(TypeName, MethodName), Ty<R>>>,
        static_methods: Box<dyn Store<(TypeName, MethodName), Ty<R>>>,
        constructors: Box<dyn Store<TypeName, Ty<R>>>,
    ) -> Self {
        Self {
            types,
            funs,
            consts,
            modules,
            properties,
            static_properties,
            methods,
            static_methods,
            constructors,
        }
    }

    /// Construct a `ShallowDeclStore` which looks up class members from the
    /// given `types` table rather than maintaining separate member stores.
    /// Intended to be used with `Store` implementations which hold on to
    /// hash-consed `Ty`s in memory (rather than storing them in a
    /// serialized format), so that looking up individual members doesn't
    /// involve deserializing an entire `ShallowClass`.
    pub fn with_no_member_stores(
        types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
        funs: Box<dyn Store<FunName, Arc<FunDecl<R>>>>,
        consts: Box<dyn Store<ConstName, Arc<ConstDecl<R>>>>,
        modules: Box<dyn Store<ModuleName, Arc<ModuleDecl<R>>>>,
    ) -> Self {
        Self {
            properties: Box::new(PropFinder {
                types: Arc::clone(&types),
            }),
            static_properties: Box::new(StaticPropFinder {
                types: Arc::clone(&types),
            }),
            methods: Box::new(MethodFinder {
                types: Arc::clone(&types),
            }),
            static_methods: Box::new(StaticMethodFinder {
                types: Arc::clone(&types),
            }),
            constructors: Box::new(ConstructorFinder {
                types: Arc::clone(&types),
            }),

            types,
            funs,
            consts,
            modules,
        }
    }

    pub fn add_decls(&self, decls: Vec<Decl<R>>) {
        for decl in decls {
            match decl {
                Decl::Class(name, decl) => self.add_class(name, Arc::new(decl)),
                Decl::Fun(name, decl) => self.funs.insert(name, Arc::new(decl)),
                Decl::Typedef(name, decl) => {
                    self.types.insert(name, TypeDecl::Typedef(Arc::new(decl)))
                }
                Decl::Const(name, decl) => self.consts.insert(name, Arc::new(decl)),
                Decl::Module(name, decl) => self.modules.insert(name, Arc::new(decl)),
            }
        }
    }

    pub fn get_type(&self, name: TypeName) -> Option<TypeDecl<R>> {
        self.types.get(name)
    }

    pub fn get_fun(&self, name: FunName) -> Option<Arc<FunDecl<R>>> {
        self.funs.get(name)
    }

    pub fn get_const(&self, name: ConstName) -> Option<Arc<ConstDecl<R>>> {
        self.consts.get(name)
    }

    pub fn get_class(&self, name: TypeName) -> Option<Arc<ShallowClass<R>>> {
        self.types.get(name).and_then(|decl| match decl {
            TypeDecl::Class(cls) => Some(cls),
            TypeDecl::Typedef(..) => None,
        })
    }

    pub fn get_typedef(&self, name: TypeName) -> Option<Arc<TypedefDecl<R>>> {
        self.types.get(name).and_then(|decl| match decl {
            TypeDecl::Typedef(td) => Some(td),
            TypeDecl::Class(..) => None,
        })
    }

    pub fn get_property_type(
        &self,
        class_name: TypeName,
        property_name: PropName,
    ) -> Option<Ty<R>> {
        self.properties.get((class_name, property_name))
    }

    pub fn get_static_property_type(
        &self,
        class_name: TypeName,
        property_name: PropName,
    ) -> Option<Ty<R>> {
        self.static_properties.get((class_name, property_name))
    }

    pub fn get_method_type(&self, class_name: TypeName, method_name: MethodName) -> Option<Ty<R>> {
        self.methods.get((class_name, method_name))
    }

    pub fn get_static_method_type(
        &self,
        class_name: TypeName,
        method_name: MethodName,
    ) -> Option<Ty<R>> {
        self.static_methods.get((class_name, method_name))
    }

    pub fn get_constructor_type(&self, class_name: TypeName) -> Option<Ty<R>> {
        self.constructors.get(class_name)
    }

    fn add_class(&self, name: TypeName, cls: Arc<ShallowClass<R>>) {
        let cid = cls.name.id();
        for prop in cls.props.iter() {
            if let Some(ty) = &prop.ty {
                self.properties.insert((cid, prop.name.id()), ty.clone())
            }
        }
        for prop in cls.static_props.iter() {
            if let Some(ty) = &prop.ty {
                self.static_properties
                    .insert((cid, prop.name.id()), ty.clone())
            }
        }
        for meth in cls.methods.iter() {
            self.methods.insert((cid, meth.name.id()), meth.ty.clone())
        }
        for meth in cls.static_methods.iter() {
            self.static_methods
                .insert((cid, meth.name.id()), meth.ty.clone())
        }
        if let Some(constructor) = &cls.constructor {
            self.constructors.insert(cid, constructor.ty.clone())
        }
        self.types.insert(name, TypeDecl::Class(cls))
    }
}

/// Looks up props from the `types` Store instead of storing them separately.
#[derive(Debug)]
struct PropFinder<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
}

impl<R: Reason> Store<(TypeName, PropName), Ty<R>> for PropFinder<R> {
    fn get(&self, (class_name, property_name): (TypeName, PropName)) -> Option<Ty<R>> {
        self.types
            .get(class_name)
            .and_then(|decl| match decl {
                TypeDecl::Class(cls) => Some(cls),
                TypeDecl::Typedef(..) => None,
            })
            .and_then(|cls| {
                cls.props.iter().rev().find_map(|prop| {
                    if prop.name.id() == property_name {
                        prop.ty.clone()
                    } else {
                        None
                    }
                })
            })
    }
    fn insert(&self, _: (TypeName, PropName), _: Ty<R>) {}
    fn remove_batch(&self, _: &mut dyn Iterator<Item = (TypeName, PropName)>) {}
}

/// Looks up props from the `types` Store instead of storing them separately.
#[derive(Debug)]
struct StaticPropFinder<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
}

impl<R: Reason> Store<(TypeName, PropName), Ty<R>> for StaticPropFinder<R> {
    fn get(&self, (class_name, property_name): (TypeName, PropName)) -> Option<Ty<R>> {
        self.types
            .get(class_name)
            .and_then(|decl| match decl {
                TypeDecl::Class(cls) => Some(cls),
                TypeDecl::Typedef(..) => None,
            })
            .and_then(|cls| {
                cls.static_props.iter().rev().find_map(|prop| {
                    if prop.name.id() == property_name {
                        prop.ty.clone()
                    } else {
                        None
                    }
                })
            })
    }
    fn insert(&self, _: (TypeName, PropName), _: Ty<R>) {}
    fn remove_batch(&self, _: &mut dyn Iterator<Item = (TypeName, PropName)>) {}
}

/// Looks up methods from the `types` Store instead of storing them separately.
#[derive(Debug)]
struct MethodFinder<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
}

impl<R: Reason> Store<(TypeName, MethodName), Ty<R>> for MethodFinder<R> {
    fn get(&self, (class_name, method_name): (TypeName, MethodName)) -> Option<Ty<R>> {
        self.types
            .get(class_name)
            .and_then(|decl| match decl {
                TypeDecl::Class(cls) => Some(cls),
                TypeDecl::Typedef(..) => None,
            })
            .and_then(|cls| {
                cls.methods.iter().rev().find_map(|meth| {
                    if meth.name.id() == method_name {
                        Some(meth.ty.clone())
                    } else {
                        None
                    }
                })
            })
    }
    fn insert(&self, _: (TypeName, MethodName), _: Ty<R>) {}
    fn remove_batch(&self, _: &mut dyn Iterator<Item = (TypeName, MethodName)>) {}
}

/// Looks up methods from the `types` Store instead of storing them separately.
#[derive(Debug)]
struct StaticMethodFinder<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
}

impl<R: Reason> Store<(TypeName, MethodName), Ty<R>> for StaticMethodFinder<R> {
    fn get(&self, (class_name, method_name): (TypeName, MethodName)) -> Option<Ty<R>> {
        self.types
            .get(class_name)
            .and_then(|decl| match decl {
                TypeDecl::Class(cls) => Some(cls),
                TypeDecl::Typedef(..) => None,
            })
            .and_then(|cls| {
                cls.static_methods.iter().rev().find_map(|meth| {
                    if meth.name.id() == method_name {
                        Some(meth.ty.clone())
                    } else {
                        None
                    }
                })
            })
    }
    fn insert(&self, _: (TypeName, MethodName), _: Ty<R>) {}
    fn remove_batch(&self, _: &mut dyn Iterator<Item = (TypeName, MethodName)>) {}
}

/// Looks up constructors from the `types` Store instead of storing them separately.
#[derive(Debug)]
struct ConstructorFinder<R: Reason> {
    types: Arc<dyn Store<TypeName, TypeDecl<R>>>,
}

impl<R: Reason> Store<TypeName, Ty<R>> for ConstructorFinder<R> {
    fn get(&self, class_name: TypeName) -> Option<Ty<R>> {
        self.types
            .get(class_name)
            .and_then(|decl| match decl {
                TypeDecl::Class(cls) => Some(cls),
                TypeDecl::Typedef(..) => None,
            })
            .and_then(|cls| cls.constructor.as_ref().map(|meth| meth.ty.clone()))
    }
    fn insert(&self, _: TypeName, _: Ty<R>) {}
    fn remove_batch(&self, _: &mut dyn Iterator<Item = TypeName>) {}
}