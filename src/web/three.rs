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

use rand::prelude::*;
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
            webGLRenderer.setClearColor(0xffffff, 1.0);
            webGLRenderer.setSize(canvas.width, canvas.height);
            webGLRenderer.shadowMap.enabled = true;
            return webGLRenderer;
        };
        let camera = js! {
            let canvas = @{canvas.as_ref()};
            let camera = new THREE.PerspectiveCamera(45, canvas.width / canvas.height, 0.1, 1000);
            camera.position.x = @{bin_spec.width} * 1.5;
            camera.position.y = @{bin_spec.height} * 1.5;
            camera.position.z = @{bin_spec.depth} * 1.5;
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
        let (mut x, mut y, mut z) = rect.center();
        x -= f64::from(self.bin_spec.width) * 0.5;
        y -= f64::from(self.bin_spec.height) * 0.5;
        z -= f64::from(self.bin_spec.depth) * 0.5;

        let item = js! {
            let scene = @{self.scene.as_ref()};
            let geo = new THREE.BoxGeometry(@{rect.width()}, @{rect.height()}, @{rect.depth()});
            let mat = new THREE.MeshBasicMaterial({
                color: @{self.rand_color()},
                transparent: true,
                opacity: 0.8,
            });

            let item = new THREE.Mesh(geo, mat);
            item.position.x = @{x};
            item.position.y = @{y};
            item.position.z = @{z};
            scene.add(item);

            return item;
        };
        self.items.borrow_mut().push(item);

        let edges = js! {
            let scene = @{self.scene.as_ref()};
            let item = new THREE.BoxGeometry(@{rect.width()}, @{rect.height()}, @{rect.depth()});
            let geo = new THREE.EdgesGeometry(item);
            let mat = new THREE.LineBasicMaterial({ color: 0x000000 });

            let edges = new THREE.LineSegments(geo, mat);
            edges.position.x = @{x};
            edges.position.y = @{y};
            edges.position.z = @{z};
            scene.add(edges);

            return edges;
        };
        self.items.borrow_mut().push(edges);
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

    fn rand_color(&self) -> u32 {
        let mut rng = thread_rng();
        let r = rng.gen_range(0, 256);
        let g = rng.gen_range(0, 256);
        let b = rng.gen_range(0, 256);
        r << 16 | g << 8 | b
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

        js! { @(no_return)
            let scene = @{self.scene.as_ref()};
            let geo = new THREE.BoxGeometry(@{width}, @{height}, @{depth});
            let mat = new THREE.MeshBasicMaterial({
                color: 0x7f7f7f,
                wireframe: true,
            });

            let bin = new THREE.Mesh(geo, mat);
            scene.add(bin);
        }
    }
}
