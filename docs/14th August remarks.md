Some remarks:

## Navigation: 
Preset view buttons. Not all people are proficient in navigating 3d environments, get lost, and need a quick "back to home" button 
(NTH) predefined "point of interest" camera locations. 
Mouse driven. Exact inputs TBD to work with the GUI. 
Different modes of rotation may be more or less intuitive. FPS style lends itself to "looking around", but not for moving around. Rotating around center point works well when zoomed out, rotating around a selected point close to the screen center is nice but has issues with points very close to the camera. 
Rotation below the XY plane is usually not necessary, and we can reduce confusion by restricting that. 
(NTH) Basic keyboard commands like "delete" or "ctrl+c, ctrl+v"; if possible even "undo" 
(NTH) only for display/presentation: touch enabled/tolerant:  
(single finger panning, pinch to zoom, dual finger rotating) 
 
## GUI: 
Tools must be toggleable in the backend. We want to be able to present a link to someone without them having the option to change the cloud.  
Current tools are quite technical 
The first four tools (lasso, knife, paint, polygon) can be combined into one. (polygon seems a good candidate) 
Users don't really need to engage with the fact that these are point clouds. They want to be able to select a car or a tree, or define a surface and modify that.  
"painting" can probably be done by selecting an option from a list and assigning that option to the selection; as there is some intelligence needed in the conversion. While initially, a solution that only changes the classification is fine, it should be setup so it can be extended to do some processing on the selection. For example: 
Street > sidewalk: 
Change of classification 
Align height with adjacent sidewalk 
Creation of curb 
Changing height of assets placed on the newly minted sidewalk 
Street > zebra crossing: 
Determining how many stripes, size, and direction 
Change classification of only stripes 
Asset selection 
(NTH) Already existing assets should be selectable (ex: cars) 
Data structure to be discussed (ex: loaded as different clouds, having an object #/scalar field/metadata that connects points into assets; can be done before the cloud is presented to you) 
(NTH) a way to change the point cloud by placing object  
Ex: a car defining parking lines under itself 
Ex: a bush changing the classification of the points below 
(NTH) thumbnails of assets in asset manager 
Import/export of assets does not need to be exposed to users. A folder on server with the assets & metadata is fine 
Import/export of clouds does not need to be exposed to users directly. 
A link with specific clouds is given to users,  
Change of workspace in GUI is not needed. 
no support for external clouds or their conversion to display data structure. 
Alterations to clouds (especially if saved and replayed as operation history; TBD) saved to server. 
Distribution of clouds in common formats is done and controlled by us manually. 
(NTH) If alterations are saved as operation history; replay can be done on heavier machines to get higher resolution versions. 
(NTH) being able to see the same cloud as modified by others. 
No simultaneous modifying needed, for display and comparison (current state vs proposed plans) only. 
(NTH) acces/edit permission control. 
Street marking tool may need to be separate, especially a polyline drawing tool. 
Full lines 
Interrupted lines 
(NTH) configurable 
Measuring tools. 
Distance between two points 
Width of classified section perpendicular on a polyline 
Ex: drawing a line along a street, and seeing its width along the line, either by displayed numbers, or ideally by changing colours. 
 
 
## Workflow: 
 fine by me. I'd love to have access to the code, more because I'm curious than because I can meaningfully add to it.  
 
## Pipeline: 
LAZ is fine as source, although we use LAS as default.  
LAS (LAZ) 1.2, 1.3, 1.4 need to be supported, as we may need other point formats to add more classification categories than the 4 bits of the 1.2 format allows. 
 
## Technology stack: 
Go for it. 
 
## Data pipeline: 
Position texture being 16 bit float may be small; as we need about millimeter accuracy on clouds that may exceed a hundred meters of length.  
I may be misinterpreting what the position texture exactly represents.  
(NTH) real color data (rgb) 
(NTH) intensity 
A nice stand-in for color information, and can be effectively used to add detail to a uniformly coloured classification. 
Eye-dome shaders do a good job of adding detail too (which they do now) 
8 bit metadata may be too small if object #, or more granular classification, etc come into the mix. 
If generating a surface heightmap helps with the interface, go for it, but determining z-offset for assets can be done satisfactorily by using neighbouring points. 
Hardware: 30FPS on 4k datasets is okay; 60 is a nice goal to go for. 
 
## Runtime architecture: 
Metadata texture seems defined here as a 32bit float ipv 16bit float? 
(NTH) Colour mapping from .las RGB values too 
 
## Backend: 
What do you need as far as servers and their configuration goes? 
We have some options using microsoft azure. 
 
## Further questions: 
We will be developing this system further when you are done with it 
Probably adding or changing UI elements and letting them do short, script-like things.  
Local storage/local server running to have a semblance of an offline application?