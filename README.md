# cap 🧢
Computer-Aided design lisP

### Status
CAP works, but is missing a lof of vital features like rotations and subtractions. It was written as a simple way of dynamically doing CAD in Minecraft, with OpenSCAD support to export the model to a more functional format.


## Features
- [x] Translate
- [x] Polygon
- [x] Extrude

## Supported targets/backends
- [x] OpenSCAD
- [x] Minecraft (generating commands + running them through RCON, see `apply-mccmd`)
- [ ] Some sort of 3D object file with support for materials/textures

## To Do
- [ ] Materials/Textures
- [ ] Rotate
- [ ] Scale
- [ ] Difference/Subtract
    - Global delete would be easy, just add "air" blocks in minecraft and a difference on everything in scad
    - "Local" delete would be harder, and only marginally more useful (might require a CSG library for the minecraft backend)
- [ ] Add translate as a built-in function
