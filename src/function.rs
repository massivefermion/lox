use std::collections::HashMap;
use std::fmt::{Debug, Display};

use crate::chunk::Chunk;
use crate::op::OpCode;
use crate::value::Value;

#[derive(Clone)]
pub(crate) struct Function {
    arity: u128,
    name: String,
    codes: Chunk<usize>,
    has_return: Option<bool>,
    captures: HashMap<String, (usize, usize, Option<Value>)>,
}

impl Function {
    pub(crate) fn new(name: String, arity: u128) -> Function {
        Function {
            name,
            arity,
            codes: Chunk::new(),
            has_return: Some(false),
            captures: HashMap::new(),
        }
    }

    pub(crate) fn new_main(name: String) -> Function {
        Function {
            name,
            arity: 0,
            has_return: None,
            codes: Chunk::new(),
            captures: HashMap::new(),
        }
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn arity(&self) -> u128 {
        self.arity
    }

    pub(crate) fn code_size(&self) -> usize {
        self.codes.size()
    }

    pub(crate) fn get_code(&self, index: usize) -> Option<usize> {
        self.codes.get(index).copied()
    }

    pub(crate) fn add_op(&mut self, op: OpCode) {
        self.codes.add(op as usize);
    }

    pub(crate) fn add_jump(&mut self, if_false: bool) -> usize {
        match if_false {
            true => self.codes.add(OpCode::JumpIfFalse as usize),
            false => self.codes.add(OpCode::Jump as usize),
        };
        self.codes.add(OpCode::Invalid as usize)
    }

    pub(crate) fn patch_jump(&mut self, address: usize) {
        self.codes.set(address, self.codes.size() - address - 1);
    }

    pub(crate) fn add_jump_back(&mut self, target: usize) {
        self.codes.add(OpCode::JumpBack as usize);
        self.codes.add(target);
    }

    pub(crate) fn add_address(&mut self, address: usize) {
        self.codes.add(address);
    }

    pub(crate) fn has_return(&self) -> Option<bool> {
        self.has_return
    }

    pub(crate) fn already_returns(&mut self) {
        self.has_return = Some(true);
    }

    pub(crate) fn captures(&self) -> HashMap<String, (usize, usize, Option<Value>)> {
        self.captures.clone()
    }

    pub(crate) fn add_capture(&mut self, name: String, frame: usize, address: usize) {
        self.captures.insert(name, (frame, address, None));
    }

    pub(crate) fn populate_capture(&mut self, name: String, value: Value) {
        if let Some((frame, address, _)) = self.captures.get(&name) {
            self.captures.insert(name, (*frame, *address, Some(value)));
        }
    }

    pub(crate) fn get_capture(&self, name: String) -> Option<Value> {
        self.captures
            .get(&name)
            .and_then(|(_, _, value)| value.clone())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.arity)
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iterator = self.codes.into_iter().peekable().enumerate();
        while let Some((offset, current)) = iterator.next() {
            let op_code = OpCode::from(*current as u8);
            let string_offset = format!("{:0>4}", offset);
            // writeln!(f, "{}   {:?}", string_offset, op_code)?;
            let params = OpCode::params(&op_code);
            for _ in 0..params {
                iterator.next();
                // let Some((offset, address)) = iterator.next() else {
                //     todo!()
                // };
                // let string_offset = format!("{:0>4}", offset);
                // writeln!(f, "{}   {:?}", string_offset, address)?;
            }
            writeln!(f, "{}   {:?}", string_offset, op_code)?;
        }
        Ok(())
    }
}
