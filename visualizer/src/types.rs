/*
 * Copyright 2019 Zejun Li
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::result::Result as StdResult;

use serde::*;

use kaosu_packer::geom::Cuboid;
use kaosu_packer::Params;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Csv(err: csv::Error) {
            from()
            description(err.description())
        }
        InputFileNotCsv {
            description("input file not a csv, please use csv file")
        }
        NoInputFile {
            description("no file selected, please select a csv file")
        }
        NotValidNumber {
            description("not a valid number, please input a valid number")
        }
        NoBoxToBePack {
            description("there is no box to be pack")
        }
        InvalidBinSpec {
            description("bin's width, depth or height must greater than 0")
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(PartialEq, Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Item {
    pub width: i32,
    pub depth: i32,
    pub height: i32,
    pub group: usize,
}

impl Into<Cuboid> for &Item {
    fn into(self) -> Cuboid {
        Cuboid::new(self.width, self.depth, self.height)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct ProblemSpec {
    pub params: Params,
    pub bin: Cuboid,
    pub items: Vec<Item>,
}

impl ProblemSpec {
    pub fn validate(&self) -> Result<()> {
        if self.items.is_empty() {
            return Err(Error::NoBoxToBePack);
        }
        if self.bin.height <= 0 || self.bin.depth <= 0 || self.bin.width <= 0 {
            return Err(Error::InvalidBinSpec);
        }
        Ok(())
    }
}
