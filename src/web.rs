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

use yew::prelude::*;

mod inner;
mod input_process;
mod three;
mod visualize;
mod worker;

use self::inner::*;
use self::input_process::InputProcess;
use self::visualize::Visualize;
use crate::Dimension;

pub use self::worker::Worker;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Msg {
    Submit(DataSpec),
    PackResult(Solution),
}

enum Page {
    InputProcess,
    Computing,
    Visualize,
}

pub struct App {
    pack_worker: Box<Bridge<worker::Worker>>,
    bin_spec: Option<Dimension>,
    pack_solution: Rc<RefCell<Solution>>,
    page: Page,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|resp: worker::Response| Msg::PackResult(resp.into()));
        let pack_worker = worker::Worker::bridge(callback);
        App {
            pack_worker,
            pack_solution: Rc::new(RefCell::new(Vec::new())),
            bin_spec: None,
            page: Page::InputProcess,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit(spec) => {
                self.bin_spec = Some(spec.bin_spec);
                self.pack_worker.send(spec.into());
                self.page = Page::Computing;
                true
            }
            Msg::PackResult(solution) => {
                self.pack_solution.replace(solution);
                self.page = Page::Visualize;
                true
            }
        }
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        match self.page {
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
                <Visualize: solution=self.pack_solution.clone(),
                            bin_spec=self.bin_spec.unwrap(),/>
            },
        }
    }
}
