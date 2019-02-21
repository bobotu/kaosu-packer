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

use serde::*;
use yew::prelude::worker::*;

use super::types::*;
use kaosu_packer::{pack_boxes, PackSolution};

pub struct Packer {
    link: AgentLink<Packer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Problem(ProblemSpec),
}

impl Transferable for Request {}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Solution(PackSolution),
}

impl Transferable for Response {}

impl Agent for Packer {
    type Reach = Public;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        Packer { link }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Request::Problem(input) => {
                let result = pack_boxes(input.params, input.bin, &input.items);
                self.link.response(who, Response::Solution(result));
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "packer.js"
    }
}
