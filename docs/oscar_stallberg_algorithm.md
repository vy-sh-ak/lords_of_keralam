Oskar Stålberg’s signature procedural generation algorithm combines an altered version of the Wave Function Collapse (WFC) algorithm with custom irregular, non-orthogonal grid systems.
Instead of generating worlds on a standard square grid, his system maps 3D assets onto distorted, organic layouts, automatically deciding how adjacent blocks should connect to form coherent structures like roofs, stairs, and arches.
------------------------------
## 1. The Core Architecture
Stålberg’s approach abandons traditional noise-based terrain generation (like Perlin noise) in favor of constraint-based puzzle solving.

* The Grid Base: He constructs a grid using distorted quadrilaterals derived from an irregular triangle mesh. This removes the mechanical "Minecraft-like" look and introduces natural, winding curves.
* The Tile Set: He designs a curated kit of modular 3D meshes (e.g., solid wall, wall with window, roof piece, archway).
* The Rules (Sockets): Every edge of every tile contains metadata (or "sockets") defining what can legally touch it. For example, a "roof top" socket cannot be placed underneath a "solid foundation" socket.

------------------------------
## 2. The Modified Wave Function Collapse (WFC)
The Wave Function Collapse algorithm, originally formulated by Maxim Gumin, is inspired by quantum mechanics. In quantum physics, a particle exists in a superposition of all possible states until it is observed and "collapses" into a single state. Stålberg adapts this logic mathematically:

* Superposition: When a world is blank, every single coordinate on the grid contains all possible tile variations simultaneously.
* The Collapse: When a player clicks to place a block (or when the game auto-generates a segment), the algorithm "observes" that specific coordinate. It forces that cell to collapse into one definitive, legal tile.
* Entropy-Driven Propagation: The collapse of one cell changes the mathematical probabilities of its neighboring cells. The algorithm evaluates the neighbors with the lowest entropy (cells with the fewest remaining legal choices) and narrows down their options.
* Chain Reaction: This reduction propagates outward through the grid like a wave, instantly eliminating illegal tile combinations across the map so the structure never breaks.

------------------------------
## 3. Handling Contradictions and Backtracking
A major mathematical hurdle in standard WFC is a "contradiction"—a dead-end scenario where a cell propagates rules that leave a neighboring cell with zero valid tile options.
Stålberg solves this dynamically to prevent the game from freezing:

* Player-Driven Correction: In a toy like Townscaper, the player provides the core constraint. If a player places a block that causes a structural contradiction, the algorithm dynamically removes blocks or "backtracks" locally, shifting neighboring tiles to alternative valid states.
* Determined Fallbacks: If a cell is backed into a corner with no legal tile, it falls back to an empty space or a default "ruin" asset rather than crashing the system.

------------------------------
## 4. Mathematical Geometry of the Grid
To break the grid monotony, Stålberg utilizes an underlying quad-dominant mesh built from a relaxed Voronoi diagram or a distorted Fibonacci sphere:
$$x^2 + y^2 + z^2 = R^2$$ 
By distorting the vertices slightly and converting triangles into four-sided polygons (quads), he creates a grid where cells have an irregular number of neighbors (sometimes 3, 5, or 6 instead of exactly 4). The WFC algorithm is robust enough to map its rules to these irregular connection points, creating the illusion of handcrafted, organic architecture.
------------------------------
## ✅ Summary of the Algorithm
Oskar Stålberg’s procedural system relies on Wave Function Collapse operating on an irregular quad mesh. It treats world-building like a real-time puzzle solver, calculating localized tile constraints instantly upon player interaction.
------------------------------
Would you like to explore a simplified Python code example of how WFC constraints propagate, or do you want to break down the geometry of his irregular meshes further?



## Steps to do it Algorithmically:

1. **The Regular Grid**: The process begins with a simple grid where points are placed at fixed distances and connected by lines to establish a basic placement system.
2. **The Dual Grid:** To achieve seamless connections with rounded corners, a dual grid was implemented. This grid is offset by half a cell, allowing types (land/water) to be defined by the corners of the cell, which reduced the required 3D model variations from 15 down to 6.
3. **Variations to Hide Repetition (06:34):** To make the environment feel less robotic, multiple aesthetic variants were created for each piece and selected randomly during placement.
4. **Grid Deformation (07:05):** To break away from the stiff, regular grid look, deformation handles were defined on the bounds of the models. By transforming vertex positions relative to these handles, the meshes could be "squashed and stretched" to fit a non-regular grid.
5. **The Stålberg Grid (08:06):** This is the final, organic evolution:
   - **Hexagonal Patching:** Points are placed in a hexagon shape, with additional layers and points between outer points.
   - **Triangulation:** Points are connected into triangles starting from the center.
   - **Dissolving Edges:** Edges are randomly dissolved to turn pairs of triangles into quads.
   - **Subdivision:** Remaining triangles are divided into quads, and then each quad is divided into four smaller ones to ensure a pure-quad mesh.
   - **Relaxation:** A relaxation algorithm is applied to iteratively adjust point positions so they are equally distant from their neighbors, resulting in a smooth, organic, "wave-like" grid that can be tiled infinitely.