# kaosu-packer

## What is this
`kaosu-packer` is a 3d bin packing problem solver with a GUI to visualize the solution.   

The packing algorithm used in this project is based on `The packing algorithm is based on J. F. Gonçalves and M. G. C. Resende, “A biased random key genetic algorithm for 2D and 3D bin packing problems,” International Journal of Production Economics, vol. 145, no. 2, pp. 500–510, Oct. 2013.`

## How To Run
You should have `Rust`, `make` and `cargo-web` installed.   
Then simply run `make` to compile and use `caddy` or any others http server to host files in `build` folder.

## Data Format
The data is given in CSV format
```
height,depth,width,count
4,4,4,20
3,2,1,10
7,3,5,10
6,7,4,20
```

## About project's name
**あばばばばばばばばば**

かおす (kaosu) is the pen name of [萌田薫子](https://zh.moegirl.org/zh-hans/萌田薰子), and it also means chaos in Japanese.   
As the BRKGA algorithm's evolution process seems a little chaos, so I choose this as project's name.
