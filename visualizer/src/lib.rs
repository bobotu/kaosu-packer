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

#![recursion_limit = "512"]

#[macro_use]
extern crate yew;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate quick_error;

mod input_process;
mod packer;
mod three;
mod types;
mod visualize;

use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;

use self::input_process::InputProcess;
pub use self::packer::Packer;
use self::types::*;
use self::visualize::Visualize;
use kaosu_packer::PackSolution;

pub enum Msg {
    Submit(Rc<RefCell<ProblemSpec>>),
    PackResult(PackSolution),
}

enum Page {
    InputProcess,
    Computing,
    Visualize,
}

pub struct App {
    pack_worker: Box<Bridge<packer::Packer>>,
    problem_spec: Option<Rc<RefCell<ProblemSpec>>>,
    pack_solution: Option<Rc<RefCell<PackSolution>>>,
    current_page: Page,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|resp: packer::Response| match resp {
            packer::Response::Solution(solution) => Msg::PackResult(solution),
        });
        let pack_worker = packer::Packer::bridge(callback);
        App {
            pack_worker,
            pack_solution: None,
            problem_spec: None,
            current_page: Page::InputProcess,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit(spec) => {
                self.pack_worker
                    .send(packer::Request::Problem(spec.borrow().clone()));
                self.problem_spec = Some(spec);
                self.current_page = Page::Computing;
                true
            }
            Msg::PackResult(solution) => {
                self.pack_solution = Some(Rc::new(RefCell::new(solution)));
                self.current_page = Page::Visualize;
                true
            }
        }
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        match self.current_page {
            Page::InputProcess => html! {
                <InputProcess: onsubmit=Msg::Submit,/>
            },
            Page::Computing => html! {
                <div id="packing",>
                    <i class="fa fa-spinner fa-5x fa-pulse fa-fw", aria-hidden="true",></i>
                    <h3>{"Packing ..."}</h3>
                </div>
            },
            Page::Visualize => html! {
                <Visualize: solution=self.pack_solution.as_ref().unwrap().clone(),
                            problem_spec=self.problem_spec.as_ref().unwrap().clone(),/>
            },
        }
    }
}
