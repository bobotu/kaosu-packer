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

use stdweb::web;
use stdweb::Value;

use crate::{Dimension, Space};
use std::cell::RefCell;

#[derive(Clone)]
pub struct ThreeRender {
    canvas: web::Element,
    scene: Value,
    render: Value,
    camera: Value,
    control: Value,
    items: RefCell<Vec<Value>>,
    bin_spec: Dimension,
}

impl ThreeRender {
    pub fn new(canvas: web::Element, bin_spec: Dimension) -> Self {
        let canvas = canvas.clone();
        let scene = js! {
            return new THREE.Scene()
        };
        let render = js! {
            let canvas = @{canvas.clone()};
            let webGLRenderer = new THREE.WebGLRenderer({
                canvas: canvas,
                antialias: true,
            });
            webGLRenderer.setSize(canvas.width, canvas.height);
            webGLRenderer.shadowMap.enabled = true;
            return webGLRenderer;
        };
        let camera = js! {
            let canvas = @{canvas.as_ref()};
            let camera = new THREE.PerspectiveCamera(45, canvas.width / canvas.height, 0.1, 1000);
            camera.position.x = @{bin_spec.width} * 2.5;
            camera.position.y = @{bin_spec.height} * 2.5;
            camera.position.z = @{bin_spec.depth} * 2.5;
            camera.lookAt(new THREE.Vector3(0, 0, 0));
            @{scene.as_ref()}.add(camera);
            return camera;
        };
        let control = js! {
            let camera = @{camera.clone()};
            let canvas = @{canvas.clone()};
            let tc = new THREE.TrackballControls(camera, canvas);
            tc.rotateSpeed = 1.0;
            tc.zoomSpeed = 1.0;
            tc.panSpeed = 1.0;
            return tc;
        };

        let three_render = ThreeRender {
            canvas,
            scene,
            render,
            camera,
            control,
            bin_spec,
            items: RefCell::new(Vec::new()),
        };
        three_render.setup();
        three_render
    }

    pub fn add_item(&self, rect: &Space) {
        let (x, y, z) = rect.center();
        let item = js! {
            let scene = @{self.scene.as_ref()};
            let geo = new THREE.BoxGeometry(@{rect.width()}, @{rect.height()}, @{rect.depth()});
            let mat = new THREE.MeshBasicMaterial({ color: 0xff0000, wireframe: true });

            let item = new THREE.Mesh(geo, mat);
            item.position.x = @{x};
            item.position.y = @{y};
            item.position.z = @{z};
            scene.add(item);

            return item;
        };
        self.items.borrow_mut().push(item);
    }

    pub fn clear(&self) {
        js! { @(no_return) @{self.control.as_ref()}.reset() };
        for item in self.items.borrow().iter() {
            js! { @(no_return)
                let scene = @{self.scene.as_ref()};
                let item = @{item};
                scene.remove(item);
                item.geometry.dispose();
                item.material.dispose();
            }
        }
        self.items.borrow_mut().clear();
    }

    fn setup(&self) {
        self.setup_light();
        self.setup_bin();
        self.setup_render_loop();
    }

    fn setup_light(&self) {
        js! { @(no_return)
            let scene = @{self.scene.as_ref()};

            let ambientLight = new THREE.AmbientLight(0xffffff);
            scene.add(ambientLight);

            let spotLight = new THREE.SpotLight(0xffffff);
            spotLight.position.set(300, 300, 300);
            spotLight.intensity = 1;
            scene.add(spotLight);
        }
    }

    fn setup_render_loop(&self) {
        js! { @(no_return)
            let scene = @{self.scene.clone()};
            let render = @{self.render.clone()};
            let camera = @{self.camera.clone()};
            let tc = @{self.control.clone()};
            let clock = new THREE.Clock();

            let r = () => {
                if (tc.screen.height === 0) {
                    tc.handleResize();
                }
                tc.update(clock.getDelta());
                render.render(scene, camera);
                requestAnimationFrame(r);
            };
            r();
        }
    }

    fn setup_bin(&self) {
        let width = self.bin_spec.width;
        let height = self.bin_spec.height;
        let depth = self.bin_spec.depth;
        let x = f64::from(width) / 2.;
        let y = f64::from(height) / 2.;
        let z = f64::from(depth) / 2.;

        js! { @(no_return)
            let scene = @{self.scene.as_ref()};
            let geo = new THREE.BoxGeometry(@{width}, @{height}, @{depth});
            let mat = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true });

            let bin = new THREE.Mesh(geo, mat);
            bin.position.x = @{x};
            bin.position.y = @{y};
            bin.position.z = @{z};
            scene.add(bin);
        }
    }
}
