# bevy_efficient_forest_example

Using bevy and custom render pipelines in order to render many objects in a forest using chunks for performance. Grass is rendered efficiently by not sending loads of data to the gpu but instead randomizing everything on the shader. The trees and the other objects are rendered using gpu instancing. 

I could render 10 million grass straws at 70fps on my gaming pc using this approach. No one should need to render this much grass but it's good to know one can :)

Rendering the grass with with the optimized grass renderer instead of the more general gpu instancing gets me 3x-4x better fps. (When only rendering a bunch of grass)

I'm making a game for the web in which you have to code the behavior of forest animals in order to balance an ecosystem. Follow my devlogs if you are interested: https://www.youtube.com/channel/UCqzbiRaNXJa50J4kvJvKpAg

Compatible Bevy versions:
- Bevy 0.8

Forest:
Total instanced objects 192_000 (Not all visable at once, culling)
Total grass straws 18_000_000 (Not all visable at once, culling)
(90fps when looking around with the current camera constraints)
(20x20 chunks):

https://user-images.githubusercontent.com/14301446/186657930-79de7f46-c1f0-4e52-b879-b182b2c9b8e2.mp4

Grass 1.4 million in 1 draw call (300fps) (entire map in only 1 chunk):

https://user-images.githubusercontent.com/14301446/186659929-6a98a0f0-8a9c-4999-9956-b55c2f37c6db.mp4

Textured Trees 115200  (60fps when looking at all chunks) (8x8 chunks)

https://user-images.githubusercontent.com/14301446/186662761-832b57ed-8096-487a-9d6f-ad7eb15ddc35.mp4

My Computer:
- Nvidia 1080 Ti
- AMD Ryzen 5 5600X 6-Core Processor
- 32Gb Ram 2666MHz
