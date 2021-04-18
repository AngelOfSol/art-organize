# CHANGELOG
## v0.3.0
* Adds searching
  * Navigating to the search screen will return a view containing the results the search bar
  * Search syntax
    * Items
      * "skeb" -> Searches for all pieces with a tag named "skeb" and any category
        * If no tag exists in your database (e.g. "my_nonexistent_tag"), it will be ignored in the search
      * "source:skeb" -> Searches for all pieces with a tag named "skeb" that has the category "source"
        * If no tag exists in your database (e.g. "my_nonexistent_tag"), it will be ignored in the search
        * Non-existent categories will cause the search to fail
        * ":skeb" will search for tags named "skeb" that have no category
      * "source:fan" -> Searches for all pieces that are id'd as fan creations
        * "source:official" and "source:commission" are the other two
      * "media:image" -> Searches for all pieces that are id'd as images
        * "media:text" exists, but is generally unused at the moment
      * "after:12/1/2020" -> Searches for all pieces added on or after December 1st 2020
        * "before:12/1/2020" does the inverse search
      * "tip>=20" -> Searches for all pieces with a tip greater than 20
        * ">=" or "<=" are the valid operations
        * "tip", "base", and "total" are the valid prices to select from
    * Modifiers
      * Negate
        * "!skeb" -> Inverts the condition of "skeb", returning pieces that don't match that tag
        * Only affects the next item
      * And
        * "skeb portrait" -> Searches for all pieces that match both "skeb" and "portrait"
        * This condition can have any number of items (e.g. "skeb portrait hat")
      * Or
        * "skeb|twitter" -> Searches for all pieces that match either "skeb" or "twitter"
        * This condition can have any number of items (e.g. "skeb|twitter|deviantart")
    * Complex Searches
      * Or binds tighter than And
        * "portrait skeb|twitter" -> Searches for all pieces that
          * match "skeb" or "twitter"
          * AND match "portrait"
      * Parens
        * Parentheses let you group search terms together to make more complex searches
        * "(portrait skeb)|twitter" -> Searches for all pieces that
          * match "portrait" and "skeb"
          * OR match just "twitter"
      * Negating a grouping
        * "!skeb portrait" -> Searches for all pieces that
          * don't match "skeb"
          * AND do match "portrait
        * "!(skeb portrait") -> Searches for all pieces that
          * don't match
            * matches "skeb" and "portrait
          * (e.g. pieces that match "!skeb !portrait" or "!skeb portrait" or "skeb !portrait")

## v0.2.0
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
  * Adding and removing tags is grouped by categories for convenience
* Tags with an associated category will be colored according to the categories color
* Tags are generally sorted by category and then by tag name (alphabetically in both cases)
* When viewing a tag, pieces with that tag will be displayed in the main window
  * This is the pre-cursor to the search functionality
* When viewing a category, tags with that category will be listed in the main window
* The grey numbers next to tags and categories display the counts
  * The amount of pieces with a given tag
  * The amount of tags with a given category
* Pieces and blobs that have unloaded images will display the name of their piece or blob respectively
* Added a new piece list screen
  * Lists out all the pieces and some relevant data
  * Contains a summary of some information on the left side
* Clipboard now works for text
* When loading images, thumbnails and raw images will be requested separately
* Copy to clipboard button added to blobs
  * Sadly, transparency will not be respected (blame windows)
* Save to file button added to blobs
* Items are now generally sorted by either their name or their date added

### Backend
* Caps FPS at ~144
 
## v0.1.0
Released initial version.

### Features
* Organize your commissions into pieces.
* Add as many images into each piece as needed, classifying them based on what part of the drawing process they're from
* Saves all the image files into a local directory associated with the database.