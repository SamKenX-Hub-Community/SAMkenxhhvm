// @generated by Thrift for thrift/compiler/test/fixtures/no-legacy-apis/src/module.thrift
// This file is probably not the place you want to edit!

//! Thrift service definitions for `module`.


/// Service definitions for `MyService`.
pub mod my_service {
    #[derive(Clone, Debug)]
    pub enum QueryExn {
        #[doc(hidden)]
        Success(crate::types::MyStruct),
        ApplicationException(::fbthrift::ApplicationException),
    }

    impl ::std::convert::From<crate::errors::my_service::QueryError> for QueryExn {
        fn from(err: crate::errors::my_service::QueryError) -> Self {
            match err {
                crate::errors::my_service::QueryError::ApplicationException(aexn) => QueryExn::ApplicationException(aexn),
                crate::errors::my_service::QueryError::ThriftError(err) => QueryExn::ApplicationException(::fbthrift::ApplicationException {
                    message: err.to_string(),
                    type_: ::fbthrift::ApplicationExceptionErrorCode::InternalError,
                }),
            }
        }
    }

    impl ::std::convert::From<::fbthrift::ApplicationException> for QueryExn {
        fn from(exn: ::fbthrift::ApplicationException) -> Self {
            Self::ApplicationException(exn)
        }
    }

    impl ::fbthrift::ExceptionInfo for QueryExn {
        fn exn_name(&self) -> &'static str {
            match self {
                Self::Success(_) => panic!("ExceptionInfo::exn_name called on Success"),
                Self::ApplicationException(aexn) => aexn.exn_name(),
            }
        }

        fn exn_value(&self) -> String {
            match self {
                Self::Success(_) => panic!("ExceptionInfo::exn_value called on Success"),
                Self::ApplicationException(aexn) => aexn.exn_value(),
            }
        }

        fn exn_is_declared(&self) -> bool {
            match self {
                Self::Success(_) => panic!("ExceptionInfo::exn_is_declared called on Success"),
                Self::ApplicationException(aexn) => aexn.exn_is_declared(),
            }
        }
    }

    impl ::fbthrift::ResultInfo for QueryExn {
        fn result_type(&self) -> ::fbthrift::ResultType {
            match self {
                Self::Success(_) => ::fbthrift::ResultType::Return,
                Self::ApplicationException(_aexn) => ::fbthrift::ResultType::Exception,
            }
        }
    }

    impl ::fbthrift::GetTType for QueryExn {
        const TTYPE: ::fbthrift::TType = ::fbthrift::TType::Struct;
    }

    impl<P> ::fbthrift::Serialize<P> for QueryExn
    where
        P: ::fbthrift::ProtocolWriter,
    {
        fn write(&self, p: &mut P) {
            if let Self::ApplicationException(aexn) = self {
                return aexn.write(p);
            }
            p.write_struct_begin("Query");
            match self {
                Self::Success(inner) => {
                    p.write_field_begin(
                        "Success",
                        ::fbthrift::TType::Struct,
                        0i16,
                    );
                    inner.write(p);
                    p.write_field_end();
                }
                Self::ApplicationException(_aexn) => unreachable!(),
            }
            p.write_field_stop();
            p.write_struct_end();
        }
    }

    impl<P> ::fbthrift::Deserialize<P> for QueryExn
    where
        P: ::fbthrift::ProtocolReader,
    {
        fn read(p: &mut P) -> ::anyhow::Result<Self> {
            static RETURNS: &[::fbthrift::Field] = &[
                ::fbthrift::Field::new("Success", ::fbthrift::TType::Struct, 0),
            ];
            let _ = p.read_struct_begin(|_| ())?;
            let mut once = false;
            let mut alt = ::std::option::Option::None;
            loop {
                let (_, fty, fid) = p.read_field_begin(|_| (), RETURNS)?;
                match ((fty, fid as ::std::primitive::i32), once) {
                    ((::fbthrift::TType::Stop, _), _) => {
                        p.read_field_end()?;
                        break;
                    }
                    ((::fbthrift::TType::Struct, 0i32), false) => {
                        once = true;
                        alt = ::std::option::Option::Some(Self::Success(::fbthrift::Deserialize::read(p)?));
                    }
                    ((ty, _id), false) => p.skip(ty)?,
                    ((badty, badid), true) => return ::std::result::Result::Err(::std::convert::From::from(
                        ::fbthrift::ApplicationException::new(
                            ::fbthrift::ApplicationExceptionErrorCode::ProtocolError,
                            format!(
                                "unwanted extra union {} field ty {:?} id {}",
                                "QueryExn",
                                badty,
                                badid,
                            ),
                        )
                    )),
                }
                p.read_field_end()?;
            }
            p.read_struct_end()?;
            alt.ok_or_else(||
                ::fbthrift::ApplicationException::new(
                    ::fbthrift::ApplicationExceptionErrorCode::MissingResult,
                    format!("Empty union {}", "QueryExn"),
                )
                .into(),
            )
        }
    }
}
