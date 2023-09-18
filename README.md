# bevy_efficient_forest_example

Using bevy and custom render pipelines in order to render many objects in a forest using chunks for performance. Grass is rendered efficiently by not sending loads of data to the gpu but instead randomizing everything on the shader. The trees and the other objects are rendered using gpu instancing. 

I could render 8 million grass straws at 60fps on my gaming pc using this approach. No one should need to render this much grass but it's good to know one can :)

Rendering the grass with with the optimized grass renderer instead of the more general gpu instancing gets me 3x-4x better fps. (When only rendering a bunch of grass)

I'm making a game for the web in which you have to code the behavior of forest animals in order to balance an ecosystem. Follow my devlogs if you are interested: https://www.youtube.com/channel/UCqzbiRaNXJa50J4kvJvKpAg

Compatible Bevy versions:
- Bevy 0.11 (main)
- Bevy 0.9 tag v0.9.1)
- Bevy 0.8 (branch bevy-0.8.1 or tag v0.1.0)

Forest:
Total instanced objects 192_000 (Not all visable at once, culling)
Total grass straws 18_000_000 (Not all visable at once, culling)
(90fps when looking around with the current camera constraints)
(20x20 chunks):

https://github.com/pinkponk/bevy_efficient_forest_rendering/assets/14301446/fb201c0b-7f27-45d0-b6c0-559be97e6677

Grass 1.28 million (150fps) (2x2 chunks):

https://github.com/pinkponk/bevy_efficient_forest_rendering/assets/14301446/e1e76388-2257-406a-bfbe-1db4e5c238af

My Computer:
- Nvidia 1080 Ti
- AMD Ryzen 5 5600X 6-Core Processor
- 32Gb Ram 2666MHz
