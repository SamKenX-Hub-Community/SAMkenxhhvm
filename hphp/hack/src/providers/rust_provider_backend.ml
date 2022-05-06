(*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *)

type t

external make_ffi :
  root:string -> hhi_root:string -> tmp:string -> ParserOptions.t -> t
  = "hh_rust_provider_backend_make"

let make popt =
  make_ffi
    ~root:Relative_path.(path_of_prefix Root)
    ~hhi_root:Relative_path.(path_of_prefix Hhi)
    ~tmp:Relative_path.(path_of_prefix Tmp)
    popt

module Decl = struct
  external get_fun : t -> string -> Shallow_decl_defs.fun_decl option
    = "hh_rust_provider_backend_get_fun"

  external get_shallow_class :
    t -> string -> Shallow_decl_defs.class_decl option
    = "hh_rust_provider_backend_get_shallow_class"

  external get_typedef : t -> string -> Shallow_decl_defs.typedef_decl option
    = "hh_rust_provider_backend_get_typedef"

  external get_gconst : t -> string -> Shallow_decl_defs.const_decl option
    = "hh_rust_provider_backend_get_gconst"

  external get_module : t -> string -> Shallow_decl_defs.module_decl option
    = "hh_rust_provider_backend_get_module"

  external get_folded_class : t -> string -> Decl_defs.decl_class_type option
    = "hh_rust_provider_backend_get_folded_class"

  external push_local_changes : t -> unit
    = "hh_rust_provider_backend_decl_provider_push_local_changes"

  external pop_local_changes : t -> unit
    = "hh_rust_provider_backend_decl_provider_pop_local_changes"
end

module File = struct
  type file_type =
    | Disk of string
    | Ide of string

  external get : t -> Relative_path.t -> file_type option
    = "hh_rust_provider_backend_file_provider_get"

  external get_contents : t -> Relative_path.t -> string option
    = "hh_rust_provider_backend_file_provider_get_contents"

  external provide_file_for_tests : t -> Relative_path.t -> string -> unit
    = "hh_rust_provider_backend_file_provider_provide_file_for_tests"

  external provide_file_for_ide : t -> Relative_path.t -> string -> unit
    = "hh_rust_provider_backend_file_provider_provide_file_for_ide"

  external provide_file_hint : t -> Relative_path.t -> file_type -> unit
    = "hh_rust_provider_backend_file_provider_provide_file_hint"

  external remove_batch : t -> Relative_path.Set.t -> unit
    = "hh_rust_provider_backend_file_provider_remove_batch"

  external push_local_changes : t -> unit
    = "hh_rust_provider_backend_file_provider_push_local_changes"

  external pop_local_changes : t -> unit
    = "hh_rust_provider_backend_file_provider_pop_local_changes"
end