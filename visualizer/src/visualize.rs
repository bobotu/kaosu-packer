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
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::cell::RefCell;
use std::rc::Rc;

use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{Element, INode};
use yew::prelude::*;
use yew::virtual_dom::VNode;

use super::three::ThreeRender;
use super::types::ProblemSpec;
use kaosu_packer::geom::Cuboid;
use kaosu_packer::{PackSolution, Params};

#[derive(PartialEq, Clone)]
pub struct Props {
    pub solution: Rc<RefCell<PackSolution>>,
    pub problem_spec: Rc<RefCell<ProblemSpec>>,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            solution: Rc::default(),
            problem_spec: Rc::new(RefCell::new(ProblemSpec {
                params: Params::default(),
                bin: Cuboid::new(0, 0, 0),
                items: Vec::new(),
            })),
        }
    }
}

pub enum Msg {
    NextBin,
    PrevBin,
}

pub struct Visualize {
    solution: Rc<RefCell<PackSolution>>,
    problem_spec: Rc<RefCell<ProblemSpec>>,
    utilization: Vec<f64>,
    current_idx: usize,
    canvas: Element,
    render: ThreeRender,
}

impl Component for Visualize {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        let (solution, problem_spec) = (props.solution, props.problem_spec);
        let bin_spec = problem_spec.borrow().bin;
        let utilization = Self::cal_utilization(solution.borrow().as_ref(), &bin_spec);
        let canvas = Self::create_canvas(480, 800);
        let render = ThreeRender::new(canvas.clone(), bin_spec);

        Visualize {
            current_idx: 0,
            solution,
            problem_spec,
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
        let canvas = VNode::VRef(canvas);
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
                    {format!("Bin: {} / {}", self.current_idx + 1, self.solution.borrow().len())}
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
                        {for solution.iter().map(|p| self.view_render_table_item(p.item_idx))}
                    </tbody>
                </table>
            </div>
        }
    }

    fn view_render_table_item(&self, idx: usize) -> Html<Self> {
        let item = &self.problem_spec.borrow().items[idx];
        html! {
            <tr class="pure-table-odd",>
                <td>{item.group}</td>
                <td>{item.width}</td>
                <td>{item.depth}</td>
                <td>{item.height}</td>
            </tr>
        }
    }

    fn render_items(&self) {
        self.render.clear();
        let solution = self.solution.borrow();
        for p in solution[self.current_idx].iter() {
            self.render.add_item(&p.space);
        }
    }

    fn cal_utilization(solution: &PackSolution, bin_spec: &Cuboid) -> Vec<f64> {
        let bin_vol = bin_spec.volume();
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
        let el = stdweb::web::document().create_element("canvas").unwrap();
        let canvas: CanvasElement = el.clone().try_into().unwrap();
        canvas.set_height(height);
        canvas.set_width(width);
        el
    }
}
