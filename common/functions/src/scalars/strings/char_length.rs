// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::NumberOperator;
use super::String2NumberFunction;

#[derive(Clone, Default)]
pub struct CharLength {}

impl NumberOperator<u64> for CharLength {
    const IS_DETERMINISTIC: bool = true;
    const MAYBE_MONOTONIC: bool = false;

    fn apply<'a>(&'a mut self, str: &'a [u8]) -> u64 {
        match std::str::from_utf8(str) {
            Ok(s) => s.chars().count() as u64,
            Err(_) => str.len() as u64,
        }
    }
}

pub type CharLengthFunction = String2NumberFunction<CharLength, u64>;
use crate::scalars::FunctionContext;
