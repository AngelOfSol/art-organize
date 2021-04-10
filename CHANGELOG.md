# CHANGELOG
## v0.2.0 (unreleased)
### Features
* Adds the ability to create tags with the following fields
  * Name
    * Names should follow these rules (validation will be added in the future):
      * Names should be unique
      * Names should be all lower case
      * Names should have no whitespace
      * Names should only contain the following characters:
        * 0-9 [any digit]
        * a-z [any lowercase ascii character]
        * _ [alternative to spaces]
        * () [used to separate other infomation e.g. hilda_(pokemon) vs hilda_(undernight) vs hilda_(undernight)_(cosplay)]
  * Description
    * Yes, I know the multiline text editor sucks
    * Yes, I know adding line breaks manually makes it look weird
    * I plan to support hyper links (twitter, pixiv, etc) in a different way, but for now they can be stored here
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
* Pieces and blobs that have unloaded images will display the name of their piece or blob respectively.

### Backend
* Caps FPS at ~144
 
## v0.1.0
Released initial version.

### Features
* Organize your commissions into pieces.
* Add as many images into each piece as needed, classifying them based on what part of the drawing process they're from
* Saves all the image files into a local directory associated with the database.