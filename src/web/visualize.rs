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

use super::inner::*;
use super::three::ThreeRender;
use crate::packer::*;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::unstable::TryInto;
use stdweb::web;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::Element;
use stdweb::web::INode;
use stdweb::web::Node;
use yew::prelude::*;
use yew::virtual_dom::VNode;

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub solution: Rc<RefCell<Solution>>,
    pub bin_spec: Dimension,
}

pub enum Msg {
    NextBin,
    PrevBin,
}

pub struct Visualize {
    solution: Rc<RefCell<Solution>>,
    utilization: Vec<f64>,
    current_idx: usize,
    canvas: Element,
    render: ThreeRender,
}

impl Component for Visualize {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        let (solution, bin_spec) = (props.solution, props.bin_spec);
        let utilization = Self::cal_utilization(solution.borrow().as_ref(), &bin_spec);
        let canvas = Self::create_canvas(480, 800);
        let render = ThreeRender::new(canvas.clone(), bin_spec);

        Visualize {
            current_idx: 0,
            solution,
            utilization,
            canvas,
            render,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PrevBin => {
                if self.current_idx == 0 {
                    false
                } else {
                    self.current_idx -= 1;
                    true
                }
            }
            Msg::NextBin => {
                if self.current_idx == self.utilization.len() - 1 {
                    false
                } else {
                    self.current_idx += 1;
                    true
                }
            }
        }
    }
}

impl Renderable<Visualize> for Visualize {
    fn view(&self) -> Html<Visualize> {
        let canvas = self.canvas.as_node().clone();
        let canvas = VNode::VRef(Node::from(canvas));
        self.render_items();
        html! {
            <main id="visualize",>
                { canvas }
                <div id="render-info",>
                    { self.view_render_ctl() }
                    { self.view_render_table() }
                </div>
            </main>
        }
    }
}

impl Visualize {
    fn view_render_ctl(&self) -> Html<Self> {
        html! {
            <div id="render-ctl",>
                <button class="pure-button pure-button-primary",
                        onclick=|_| Msg::PrevBin,>
                    {"Prev Bin"}
                </button>
                <span>
                    {format!("Curr Bin: {}", self.current_idx)}
                </span>
                <span>
                    {format!("Utilization: {:.2}%", self.utilization[self.current_idx])}
                </span>
                <button class="pure-button pure-button-primary",
                        onclick=|_| Msg::NextBin,>
                    {"Next Bin"}
                </button>
            </div>
        }
    }

    fn view_render_table(&self) -> Html<Self> {
        let solution = &self.solution.borrow()[self.current_idx];
        html! {
            <div class="table-wrapper",>
                <table class="pure-table",>
                    <thead>
                    <tr>
                        <th>{"Group"}</th>
                        <th>{"Width"}</th>
                        <th>{"Depth"}</th>
                        <th>{"Height"}</th>
                    </tr>
                    </thead>

                    <tbody>
                    {for solution.iter().map(|p| {
                        html! {
                            <tr class="pure-table-odd",>
                                <td>{p.item.group}</td>
                                <td>{p.item.width}</td>
                                <td>{p.item.depth}</td>
                                <td>{p.item.height}</td>
                            </tr>
                        }
                    })}
                    </tbody>
                </table>
            </div>
        }
    }

    fn render_items(&self) {
        self.render.clear();
        let solution = self.solution.borrow();
        for p in solution[self.current_idx].iter() {
            self.render.add_item(&p.space);
        }
    }

    fn cal_utilization(solution: &Solution, bin_spec: &Dimension) -> Vec<f64> {
        let bin_vol = bin_spec.width * bin_spec.height * bin_spec.depth;
        solution
            .iter()
            .map(|items| {
                let vol_used: i32 = items
                    .iter()
                    .map(|i| i.space.width() * i.space.height() * i.space.depth())
                    .sum();
                (f64::from(vol_used) / f64::from(bin_vol)) * 100.0
            })
            .collect()
    }

    fn create_canvas(height: u32, width: u32) -> Element {
        let el = web::document().create_element("canvas").unwrap();
        let canvas: CanvasElement = el.clone().try_into().unwrap();
        canvas.set_height(height);
        canvas.set_width(width);
        el
    }
}
