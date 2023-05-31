use eyre::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::transmute;
use web3::ethabi::{Address, Bytes, FixedBytes, Int, Uint};
use web3::ethabi::{Contract, Param, ParamType, StateMutability, Token};
use web3::types::{H160, U256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCall {
    name: String,
    params: HashMap<String, CallParameter>,
    state_mutability: StateMutability,
}

impl ContractCall {
    pub fn new(
        name: String,
        params: HashMap<String, CallParameter>,
        state_mutability: StateMutability,
    ) -> Self {
        Self {
            name,
            params,
            state_mutability,
        }
    }

    pub fn from_inputs(contract: &Contract, input_data: &[u8]) -> Result<ContractCall> {
        let function = match contract
            .functions()
            .find(|function| function.short_signature() == input_data[..4])
        {
            Some(function) => function,
            None => {
                return Err(eyre!("could not find function"));
            }
        };

        let mut parameters: HashMap<String, CallParameter> = HashMap::new();

        let parameter_values = match function.decode_input(&input_data[4..]) {
            Ok(values) => values,
            Err(e) => {
                return Err(eyre!("could not decode input: {:?}", e));
            }
        };

        for (parameter, value) in function.inputs.iter().zip(parameter_values) {
            parameters.insert(
                parameter.name.clone(),
                CallParameter::new(
                    parameter.name.clone(),
                    value.into(),
                    SerializableParamType::from_ethabi(parameter.kind.clone()),
                    SerializableParam::from_ethabi(parameter.clone()),
                ),
            );
        }

        Ok(Self::new(
            function.name.clone(),
            parameters,
            function.state_mutability,
        ))
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_params(&self) -> HashMap<String, CallParameter> {
        self.params.clone()
    }

    pub fn get_param(&self, name: &str) -> Result<&CallParameter> {
        self.params.get(name).ok_or_else(|| {
            eyre!(
                "param {} does not exist in function {}",
                name,
                self.get_name()
            )
        })
    }

    pub fn get_state_mutability(&self) -> StateMutability {
        self.state_mutability.clone()
    }
}
/// Function and event param types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableParamType {
    /// Address.
    Address,
    /// Bytes.
    Bytes,
    /// Signed integer.
    Int(usize),
    /// Unsigned integer.
    Uint(usize),
    /// Boolean.
    Bool,
    /// String.
    String,
    /// Array of unknown size.
    Array(Box<SerializableParamType>),
    /// Vector of bytes with fixed size.
    FixedBytes(usize),
    /// Array with fixed size.
    FixedArray(Box<SerializableParamType>, usize),
    /// Tuple containing different types
    Tuple(Vec<SerializableParamType>),
}
impl SerializableParamType {
    pub fn from_ethabi(param_type: ParamType) -> Self {
        unsafe { transmute(param_type) }
    }
}
/// Function param.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializableParam {
    /// Param name.
    pub name: String,
    /// Param type.
    pub kind: SerializableParamType,
    /// Additional Internal type.
    pub internal_type: Option<String>,
}
impl SerializableParam {
    pub fn from_ethabi(param: Param) -> Self {
        unsafe { transmute(param) }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallParameter {
    name: String,
    value: SerializableToken,
    param_type: SerializableParamType,
    inner: SerializableParam,
}

impl CallParameter {
    pub fn new(
        name: String,
        value: SerializableToken,
        param_type: SerializableParamType,
        inner: SerializableParam,
    ) -> Self {
        Self {
            name,
            value,
            param_type,
            inner,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_value(&self) -> SerializableToken {
        self.value.clone()
    }

    pub fn get_param_type(&self) -> SerializableParamType {
        self.param_type.clone()
    }
    pub fn get_inner(&self) -> SerializableParam {
        self.inner.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SerializableToken {
    Address(Address),
    FixedBytes(FixedBytes),
    Bytes(Bytes),
    Int(Int),
    Uint(Uint),
    Bool(bool),
    String(String),
    FixedArray(Vec<SerializableToken>),
    Array(Vec<SerializableToken>),
    Tuple(Vec<SerializableToken>),
}

impl From<Token> for SerializableToken {
    fn from(token: Token) -> Self {
        match token {
            Token::Address(address) => SerializableToken::Address(address),
            Token::FixedBytes(fixed_bytes) => SerializableToken::FixedBytes(fixed_bytes),
            Token::Bytes(bytes) => SerializableToken::Bytes(bytes),
            Token::Int(int) => SerializableToken::Int(int),
            Token::Uint(uint) => SerializableToken::Uint(uint),
            Token::Bool(b) => SerializableToken::Bool(b),
            Token::String(s) => SerializableToken::String(s),
            Token::FixedArray(arr) => SerializableToken::FixedArray(
                arr.into_iter().map(SerializableToken::from).collect(),
            ),
            Token::Array(arr) => {
                SerializableToken::Array(arr.into_iter().map(SerializableToken::from).collect())
            }
            Token::Tuple(tup) => {
                SerializableToken::Tuple(tup.into_iter().map(SerializableToken::from).collect())
            }
        }
    }
}

impl SerializableToken {
    pub fn into_address(&self) -> Result<H160> {
        match self {
            SerializableToken::Address(address) => Ok(*address),
            _ => Err(eyre!("invalid conversion attempt to H160")),
        }
    }

    pub fn into_fixed_bytes(&self) -> Result<Vec<u8>> {
        match self {
            SerializableToken::FixedBytes(bytes) => Ok(bytes.clone()),
            _ => Err(eyre!("invalid conversion attempt to Vec<u8>")),
        }
    }

    pub fn into_bytes(&self) -> Result<Vec<u8>> {
        match self {
            SerializableToken::Bytes(bytes) => Ok(bytes.clone()),
            _ => Err(eyre!("invalid conversion attempt to Vec<u8>")),
        }
    }

    pub fn into_int(&self) -> Result<U256> {
        match self {
            SerializableToken::Int(int) => Ok(*int),
            _ => Err(eyre!("invalid conversion attempt to U256")),
        }
    }

    pub fn into_uint(&self) -> Result<U256> {
        match self {
            SerializableToken::Uint(uint) => Ok(*uint),
            _ => Err(eyre!("invalid conversion attempt to U256")),
        }
    }

    pub fn into_bool(&self) -> Result<bool> {
        match self {
            SerializableToken::Bool(b) => Ok(*b),
            _ => Err(eyre!("invalid conversion attempt to bool")),
        }
    }

    pub fn into_string(&self) -> Result<String> {
        match self {
            SerializableToken::String(s) => Ok(s.clone()),
            _ => Err(eyre!("invalid conversion attempt to String")),
        }
    }

    pub fn into_fixed_array(&self) -> Result<Vec<SerializableToken>> {
        match self {
            SerializableToken::FixedArray(arr) => Ok(arr.clone()),
            _ => Err(eyre!(
                "invalid conversion attempt to Vec<SerializableToken>"
            )),
        }
    }

    pub fn into_array(&self) -> Result<Vec<SerializableToken>> {
        match self {
            SerializableToken::Array(arr) => Ok(arr.clone()),
            _ => Err(eyre!(
                "invalid conversion attempt to Vec<SerializableToken>"
            )),
        }
    }

    pub fn into_tuple(&self) -> Result<Vec<SerializableToken>> {
        match self {
            SerializableToken::Tuple(tuple) => Ok(tuple.clone()),
            _ => Err(eyre!(
                "invalid conversion attempt to Vec<SerializableToken>"
            )),
        }
    }
}

impl TryInto<H160> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<H160> {
        match self {
            SerializableToken::Address(address) => Ok(address),
            _ => Err(eyre!("invalid conversion attempt to H160")),
        }
    }
}

impl TryInto<Vec<u8>> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<Vec<u8>> {
        match self {
            SerializableToken::FixedBytes(bytes) => Ok(bytes),
            SerializableToken::Bytes(bytes) => Ok(bytes),
            _ => Err(eyre!("invalid conversion attempt to Vec<u8>")),
        }
    }
}

impl TryInto<U256> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<Uint> {
        match self {
            SerializableToken::Int(int) => Ok(int),
            SerializableToken::Uint(uint) => Ok(uint),
            _ => Err(eyre!("invalid conversion attempt to U256")),
        }
    }
}

impl TryInto<bool> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<bool> {
        match self {
            SerializableToken::Bool(b) => Ok(b),
            _ => Err(eyre!("invalid conversion attempt to bool")),
        }
    }
}

impl TryInto<String> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<String> {
        match self {
            SerializableToken::String(s) => Ok(s),
            _ => Err(eyre!("invalid conversion attempt to String")),
        }
    }
}

impl TryInto<Vec<SerializableToken>> for SerializableToken {
    type Error = eyre::Report;

    fn try_into(self) -> Result<Vec<SerializableToken>> {
        match self {
            SerializableToken::FixedArray(arr) => Ok(arr),
            SerializableToken::Array(arr) => Ok(arr),
            SerializableToken::Tuple(tuple) => Ok(tuple),
            _ => Err(eyre!(
                "invalid conversion attempt to Vec<SerializableToken>"
            )),
        }
    }
}
