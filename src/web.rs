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
mod worker;

use crate::packer::Placement;

use self::inner::*;
use self::input_process::*;

pub use self::worker::Worker;

pub enum Msg {
    Submit(DataSpec),
    PackResult(Vec<Vec<Placement<Item>>>),
}

enum Page {
    InputProcess,
    Computing,
    Visualize,
}

pub struct App {
    pack_worker: Box<Bridge<worker::Worker>>,
    pack_solution: Vec<Vec<Placement<Item>>>,
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
            pack_solution: Vec::new(),
            page: Page::InputProcess,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit(spec) => {
                self.pack_worker.send(spec.into());
                self.page = Page::Computing;
                true
            }
            Msg::PackResult(solution) => {
                self.pack_solution = solution;
                self.page = Page::Visualize;
                true
            }
        }
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        match self.page {
            Page::InputProcess => html! { <InputProcess: onsubmit=Msg::Submit,/> },
            Page::Computing => html! {
                <div id="packing",>
                    <i class="fa fa-spinner fa-5x fa-pulse fa-fw", aria-hidden="true",></i>
                    <h3>{"Packing ..."}</h3>
                </div>
            },
            Page::Visualize => html! {
                <p> { format!{"{:?}", self.pack_solution} } </p>
            },
        }
    }
}
