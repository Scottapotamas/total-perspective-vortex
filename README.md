# delta-planner
Path planning and generation tooling for the delta robot



# Creating splines in Blender

Using 2.79.

1. Create a project/scene.
2. In lower bar, Add -> NURBS Curve
3. Modify the control points in 3D space, add more points with the "Extrude" button in the sidebar.
4. To manipulate points, right click on the point to select it, then drag.
5. In the right-hand sidebar, select the NURBS curve, click the object data button (looks like a curve) 
6. Increase the resolution to some minimum amount (like 4). Add Bevel Depth (something like 0.001).
7. In the materials tab, add a new material. Add surface emission (white or whatever), and volume (emission) might be needed?
8. Render with Cycles, and you should see white lines following the spline.

A pair of simplistic examples are in the `/assets` for reference.