# CHANGELOG
## v0.2.0
### Features
* Adds the ability to create tags with the following fields
  * Name
  * Description
  * Date Added
  * Optionally, a category
* Adds the ability to create categories with the following fields
  * Name
  * Description
  * Date Added
  * Color
* Adds the ability to add/remove tags to/from pieces
* Tags with an associated category will be colored according to the categories color
* Tags are generally sorted by category and then by tag name (alphabetically in both cases)
* When viewing a tag, pieces with that tag will be displayed in the main window
  * This is the pre-cursor to the search functionality
* When viewing a category, tags with that category will be listed in the main window
* The grey numbers next to tags and categories display the counts
  * The amount of pieces with a given tag
  * The amount of tags with a given category

### Backend
* Caps FPS at ~144
 
## v0.1.0
Released initial version.

### Features
* Organize your commissions into pieces.
* Add as many images into each piece as needed, classifying them based on what part of the drawing process they're from
* Saves all the image files into a local directory associated with the database.