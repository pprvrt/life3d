# life3d

Simple Conway's Game of Life in Rust with Glium.

The goal of the project was to write and learn some Rust while learning a bit about OpenGL concepts in the process, even though Glium hides a lot of its complexity. All the animations (cell rotation and colors) are carried out by the vertex and fragment shaders, while the game engine only prepares the model/view/perspective matrices.

The vertex shader applies a bouncing effect on nascent cells, and a smoothstep'ed scale down on dying cells, while the fragment shaders shows nascent cells as green, and dying cells as red, all while applying an overkill Phong shading. 

# How it works

![life3d](./resources/life3d.gif)

At the beginning of a generation, the universe brings to life new cells and kills existing cells, following the rules of the Game of Life:
* Any living cell with less than two neighbours dies (underpopulation);
* Any living cell with two to three neighbours lives;
* Any living cell with more than three neighbours dies (overpopulation);
* Any dead cell with three neighbours becomes live (reproduction).

During the lifecycle of a generation, the engine steps to animate their birth and death. When the engine frame counter reaches the end of a lifecycle, a new generation starts.

The engine can be paused with `Space`. The background gets red and new generations are stopped until the engine is resumed. The engine can be resumed by pressing `Space` again. Pressing `R` will create a new random universe (p=0.5), and pressing `Del` will kill all existing cells. Speeding up and slowing down can be achieved by pressing `Right` and `Left`. Camera can be zoomed in and zoomed out with the mouse wheel.

Cells can be drawn using the mouse. The mouse is raycasted to the 3D plan (z=0) on which the cells are drawn, following the principles explained on [Mouse Picking with Ray Casting](https://antongerdelan.net/opengl/raycasting.html).

# Known issues&ramblings

Mac dropped support of OpenGL at 4.1 for Metal, but still, it should work. However on some Macs the program won't run and segfaults and I have yet to get a dump to understand why. 

Also found an interesting behaviour that requires some digging, where vertex shader GLSL code `mod(gl_InstanceID, u_width)` would somehow return `u_width`. Fixed it by replacing the modulo code by `gl_InstanceID - u_width*floor(gl_InstanceID/u_width)` which is supposedly how mod() is implemented in the first place.

# Usage

`$ cargo run --release`