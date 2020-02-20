#[macro_export]
macro_rules! link_zome_api {
    (
        $(
            $(#[$meta:meta])*
            $internal_name:literal, $enum_variant:ident, $function_name:path ;
        )*
    ) => {

        use crate::nucleus::{
            actions::{trace_invoke_wasm_api_function::trace_invoke_wasm_api_function, trace_return_wasm_api_function::trace_return_wasm_api_function},
            WasmApiFnCall,
        };
        use std::convert::TryInto;
        use crate::wasm_engine::runtime::WasmCallData;
        use holochain_json_api::json::JsonString;
        use $crate::holochain_wasmer_host::AllocationPtr;

        /// Enumeration of all the Zome Functions known and usable in Zomes.
        /// Enumeration can convert to str.
        #[repr(usize)]
        #[derive(FromPrimitive, Clone, Hash, Debug, PartialEq, Eq, Serialize)]
        pub enum ZomeApiFunction {
            /// Error index for unimplemented functions
            MissingNo = 0,

            /// Abort is a way to receive useful debug info from
            /// assemblyscript memory allocators
            /// message: mem address in the wasm memory for an error message
            /// filename: mem address in the wasm memory for a filename
            /// line: line number
            /// column: column number
            Abort,

            $(
                $(#[$meta])*
                $enum_variant
            ),*
        }

        impl Defn for ZomeApiFunction {
            fn as_str(&self) -> &'static str {
                match *self {
                    ZomeApiFunction::MissingNo => "",
                    ZomeApiFunction::Abort => "abort",
                    $(ZomeApiFunction::$enum_variant => $internal_name),*
                }
            }

            fn str_to_index(s: &str) -> usize {
                ZomeApiFunction::from_str(s).unwrap_or(ZomeApiFunction::MissingNo) as usize
            }

            fn from_index(i: usize) -> Self {
                FromPrimitive::from_usize(i).unwrap_or(ZomeApiFunction::MissingNo)
            }
        }

        impl FromStr for ZomeApiFunction {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "abort" => Ok(ZomeApiFunction::Abort),
                    $($internal_name => Ok(ZomeApiFunction::$enum_variant),)*
                    _ => Err("Cannot convert string to ZomeApiFunction"),
                }
            }
        }

        impl ZomeApiFunction {
            // cannot test this because PartialEq is not implemented for fns
            #[cfg_attr(tarpaulin, skip)]
            pub fn apply(&self, runtime: &mut Runtime, encoded_args: AllocationPtr) -> ZomeApiResult {
                // TODO Implement a proper "abort" function for handling assemblyscript aborts
                // @see: https://github.com/holochain/holochain-rust/issues/324

                match *self {
                    ZomeApiFunction::MissingNo => Ok(()),
                    ZomeApiFunction::Abort => Ok(()),
                    $( ZomeApiFunction::$enum_variant => {
                        let parameters = runtime.load_json_string_from_args(encoded_args);
                        if let Ok(context) = runtime.context() {
                            if let WasmCallData::ZomeCall(zome_call_data) = runtime.data.clone() {
                                let zome_api_call = zome_call_data.call;
                                let wasm_api_fn_call = WasmApiFnCall { function: self.clone(), parameters: parameters.clone() };
                                trace_invoke_wasm_api_function(zome_api_call.clone(), wasm_api_fn_call.clone(), &context);
                                let result = $function_name(runtime, parameters.try_into()?);
                                let wasm_api_fn_result = Ok(JsonString::from("TODO"));
                                trace_return_wasm_api_function(zome_api_call.clone(), wasm_api_fn_call, wasm_api_fn_result, &context);
                                result
                            } else {
                                error!("Can't record zome call wasm_api invocations for non zome call");
                                $function_name(runtime, parameters.try_into()?)
                            }
                        } else {
                            error!("Could not get context for runtime");
                            $function_name(runtime, parameters.try_into()?)
                        }
                    } , )*
                }
            }
        }
    };
}
