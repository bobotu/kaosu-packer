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

use std::cell::RefCell;
use std::error::Error;
use std::iter;
use std::rc::Rc;
use std::str::FromStr;

use serde::*;
use stdweb::{unstable::*, web::*};
use yew::prelude::*;

use super::types::{Error::*, *};
use kaosu_packer::geom::{Cuboid, RotationType};
use kaosu_packer::Params;

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub onsubmit: Option<Callback<Rc<RefCell<ProblemSpec>>>>,
}

pub enum Msg {
    SelectFile,
    ItemsLoaded(Result<(Vec<Item>, String)>),
    UpdateBinWidth(ChangeData),
    UpdateBinDepth(ChangeData),
    UpdateBinHeight(ChangeData),
    UpdatePopFactor(ChangeData),
    UpdateElitesPer(ChangeData),
    UpdateMutantsPer(ChangeData),
    UpdateProb(ChangeData),
    UpdateMaxGen(ChangeData),
    UpdateMaxGenNoImprove(ChangeData),
    UpdateRotation,
    Submit,
}

pub struct InputProcess {
    problem_spec: Rc<RefCell<ProblemSpec>>,
    file_name: String,
    link: ComponentLink<Self>,
    onsubmit: Callback<Rc<RefCell<ProblemSpec>>>,
}

impl Component for InputProcess {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        InputProcess {
            problem_spec: Rc::new(RefCell::new(ProblemSpec {
                params: Params::default(),
                bin: Cuboid::new(0, 0, 0),
                items: Vec::new(),
            })),
            file_name: String::new(),
            onsubmit: props.onsubmit.unwrap(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match self.inner_update(msg) {
            Err(err) => {
                stdweb::web::alert(err.description());
                false
            }
            Ok(ret) => ret,
        }
    }
}

impl InputProcess {
    fn inner_update(&mut self, msg: Msg) -> Result<bool> {
        match msg {
            Msg::ItemsLoaded(result) => {
                let (items, name) = result?;
                self.problem_spec.borrow_mut().items = items;
                self.file_name = name;
                Ok(true)
            }
            Msg::SelectFile => {
                self.read_and_parse_csv()?;
                Ok(false)
            }
            Msg::UpdateBinWidth(s) => {
                let bin_spec = &mut self.problem_spec.borrow_mut().bin;
                bin_spec.width = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateBinDepth(s) => {
                let bin_spec = &mut self.problem_spec.borrow_mut().bin;
                bin_spec.depth = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateBinHeight(s) => {
                let bin_spec = &mut self.problem_spec.borrow_mut().bin;
                bin_spec.height = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdatePopFactor(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.population_factor = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateElitesPer(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.elites_percentage = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateMutantsPer(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.max_generations = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateProb(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.inherit_elite_probability = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateMaxGen(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.max_generations = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateMaxGenNoImprove(s) => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.max_generations_no_improvement = parse_number(s)?;
                Ok(false)
            }
            Msg::UpdateRotation => {
                let params = &mut self.problem_spec.borrow_mut().params;
                params.box_rotation_type = match params.box_rotation_type {
                    RotationType::ThreeDimension => RotationType::TwoDimension,
                    RotationType::TwoDimension => RotationType::ThreeDimension,
                };
                Ok(false)
            }
            Msg::Submit => {
                self.problem_spec.borrow().validate()?;
                self.onsubmit.emit(self.problem_spec.clone());
                Ok(false)
            }
        }
    }

    fn read_and_parse_csv(&mut self) -> Result<()> {
        let input = document().get_element_by_id("file-input").unwrap();
        let files: FileList = js!(return @{input}.files).try_into().unwrap();
        match files.iter().nth(0) {
            None => Err(NoInputFile),
            Some(ref file) if !file.name().ends_with(".csv") => Err(InputFileNotCsv),
            Some(file) => {
                let callback = self.link.send_back(Msg::ItemsLoaded);
                let file1 = file.clone();
                let callback =
                    move |content: String| callback.emit(parse_csv(&content, file1.name()));
                js! { @(no_return)
                    let callback = @{callback};
                    let file_reader = new FileReader();
                    file_reader.onload = () => {
                        callback(file_reader.result);
                        callback.drop();
                    };
                    file_reader.readAsText(@{file});
                }
                Ok(())
            }
        }
    }
}

fn parse_csv(content: &str, name: String) -> Result<(Vec<Item>, String)> {
    let mut items = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(content.as_bytes());

    for (id, spec) in rdr.deserialize().enumerate() {
        let spec: BoxGroup = spec?;
        items.extend(iter::repeat(spec.into_item(id)).take(spec.count));
    }

    Ok((items, name))
}

fn parse_number<T: FromStr>(raw: ChangeData) -> Result<T> {
    let str = match raw {
        ChangeData::Value(s) => s,
        _ => unreachable!(),
    };
    match str.parse() {
        Ok(num) => Ok(num),
        _ => Err(NotValidNumber),
    }
}

#[derive(Deserialize, Copy, Clone)]
struct BoxGroup {
    width: i32,
    depth: i32,
    height: i32,
    count: usize,
}

impl BoxGroup {
    fn into_item(self, id: usize) -> Item {
        Item {
            width: self.width,
            depth: self.depth,
            height: self.height,
            group: id,
        }
    }
}

impl Renderable<InputProcess> for InputProcess {
    fn view(&self) -> Html<InputProcess> {
        html! {
            <main id="input-process",>
                <div id="data-input",>
                    { self.view_container_spec() }
                    { self.view_csv_picker() }
                </div>

                <div id="params-setter",>
                    { self.view_params_setter() }
                </div>

                <button id="run-btn",
                        onclick=|_| Msg::Submit,
                        class="pure-button pure-button-primary",>{"Run"}</button>
            </main>
        }
    }
}

impl InputProcess {
    fn view_container_spec(&self) -> Html<Self> {
        html! {
            <form id="container-spec", class="pure-form pure-form-aligned",>
                <fieldset>
                    <div class="pure-control-group",>
                        <label for="bin-width",>{"Bin Width"}</label>
                        <input id="bin-width",
                               onchange=|s| Msg::UpdateBinWidth(s),
                               type="number", min="1",
                               required="",/>
                    </div>
                    <div class="pure-control-group",>
                        <label for="bin-depth",>{"Bin Depth"}</label>
                        <input id="bin-depth",
                               onchange=|s| Msg::UpdateBinDepth(s),
                               type="number", min="0",
                               required="",/>
                    </div>
                    <div class="pure-control-group",>
                        <label for="bin-height",>{"Bin Height"}</label>
                        <input id="bin-height",
                               onchange=|s| Msg::UpdateBinHeight(s),
                               type="number", min="0",
                               required="",/>
                    </div>
                </fieldset>
            </form>
        }
    }

    fn view_csv_picker(&self) -> Html<Self> {
        html! {
            <div id="csv-picker",>
                <div id="file-input-wrapper",>
                    <label>
                        <input id="file-input",
                               onchange=|_| Msg::SelectFile,
                               type="file", required="",/>
                    </label>
                </div>
                <h5>{self.show_file_name()}</h5>
            </div>
        }
    }

    fn show_file_name(&self) -> String {
        if self.file_name.is_empty() {
            "Select the CSV of boxes".to_owned()
        } else {
            format!("Selected: {}", self.file_name)
        }
    }

    fn view_params_setter(&self) -> Html<Self> {
        let row1 = html! {
            <div style="width: 100%",>
                <div class="pure-u-1-3",>
                    <label for="population-size",>{"Population Factor"}</label>
                    <input id="population-size",
                           value=self.problem_spec.borrow().params.population_factor,
                           onchange=|s| Msg::UpdatePopFactor(s),
                           type="number", min="1",
                           required="",/>
                </div>
                <div class="pure-u-1-3",>
                    <label for="num-elites",>{"Elites Percentage"}</label>
                    <input id="num-elites",
                           value=self.problem_spec.borrow().params.elites_percentage,
                           onchange=|s| Msg::UpdateElitesPer(s),
                           type="number", min="0", step="any",
                           required="",/>
                </div>
                <div class="pure-u-1-3",>
                    <label for="num-mutants",>{"Mutants Percentage"}</label>
                    <input id="num-mutants",
                           value=self.problem_spec.borrow().params.mutants_percentage,
                           onchange=|s| Msg::UpdateMutantsPer(s),
                           type="number", min="0", step="any",
                           required="",/>
                </div>
            </div>
        };
        let row2 = html! {
            <div style="width: 100%",>
                <div class="pure-u-1-3",>
                    <label for="inherit-probability",>{"Inherit Probability"}</label>
                    <input id="inherit-probability",
                           value=self.problem_spec.borrow().params.inherit_elite_probability,
                           onchange=|s| Msg::UpdateProb(s),
                           type="number", min="0", step="any",
                           required="",/>
                </div>
                <div class="pure-u-1-3",>
                    <label for="max-gen",>{"Max Generations"}</label>
                    <input id="max-gen",
                           value=self.problem_spec.borrow().params.max_generations,
                           onchange=|s| Msg::UpdateMaxGen(s),
                           type="number", min="0",
                           required="",/>
                </div>
                <div class="pure-u-1-3",>
                    <label for="max-gen-no-improve",>{"Max No Improvement"}</label>
                    <input id="max-gen-no-improve",
                           value=self.problem_spec.borrow().params.max_generations_no_improvement,
                           onchange=|s| Msg::UpdateMaxGenNoImprove(s),
                           type="number", min="0",
                           required="",/>
                </div>
            </div>
        };
        let row3 = html! {
            <div class="pure-u-1", style="text-align: center",>
                <label for="rotate-3d", class="pure-checkbox",>
                    <input id="rotate-3d",
                           onchange=|_| Msg::UpdateRotation,
                           checked=self.is_rotate_3d(),
                           type="checkbox",/>
                    {" Rotate Boxes 3D"}
                </label>
            </div>
        };
        html! {
            <form class="pure-form pure-form-stacked",>
                <fieldset>
                    <div class="pure-g",>
                        { row1 }
                        { row2 }
                        { row3 }
                    </div>
                </fieldset>
            </form>
        }
    }

    fn is_rotate_3d(&self) -> bool {
        match self.problem_spec.borrow().params.box_rotation_type {
            RotationType::ThreeDimension => true,
            _ => false,
        }
    }
}
