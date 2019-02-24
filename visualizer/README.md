# Visualizer
This visualizer is built with WebAssembly and [yew](https://github.com/DenisKolodin/yew).

Due to the limitations of WebAssembly, the packing algorithm is single-threaded. So it may takes a while to pack for large problem set. 

## How To Run
After install `make` and `cargo-web` simply run `make` to build and use `caddy` or any others http server to serve files in `build` folder.

## Data Format
You can input problem set in this format:
```
height,depth,width,count
4,4,4,20
3,2,1,10
7,3,5,10
6,7,4,20
```
