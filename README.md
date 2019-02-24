# kaosu-packer
`kaosu-packer` is a 3d bin packing problem solver based on `The packing algorithm is based on J. F. Gonçalves and M. G. C. Resende, “A biased random key genetic algorithm for 2D and 3D bin packing problems,” International Journal of Production Economics, vol. 145, no. 2, pp. 500–510, Oct. 2013.`

There is also a WebAssembly based solution [visualizer](./visualizer). 

## Crate Features
* `serde`  enables serialization for some types, via Serde.
* `rayon` enables parallel computation in the genetic algorithm. This feature is enabled by default, and you can disable it by setting `default-features = false` in your `Cargo.toml`.

## About the project's name
**あばばばばばばばばば**

かおす (kaosu) is the pen name of [萌田薫子](https://zh.moegirl.org/zh-hans/萌田薰子), and it also means chaos in Japanese.   
As the BRKGA algorithm's evolution process seems a little chaos, so I choose this as the project's name.
