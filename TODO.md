TODO: This file is not up to date. Sorry.


## In Progress
- [ ] Use BakedExpression where sensible
	- [x] Use BakedExpression for text position, and bounding box

## TODO
- [ ] Extract shared rendering functions
- [ ] Merge goto next&prev page into turn page with direction
- [ ] Decide on http response for page changes
- [ ] Fix default for bounding boxes
- [ ] Change config format
	- [ ] Add general config section
	- [ ] Add support for multiple "pages"
	- [ ] Add support for default values for variables
- [ ] Add visual countdown example
- [ ] Add more position modes, e.g negative values for "from bottom/right"
- [ ] Add clipping at buffer edges
	- [x] Add clipping for text
- [ ] Add support for animated images
- [ ] Auto reloading on config
- [ ] Make config loading more robust
- [ ] Improve Debug and/or Display traits for elements
- [ ] Fix alpha handling
- [ ] Allow parameters for windows
- [ ] Put profiling behind feature flag
- [ ] Put profiling behind command line flag
- [ ] Fix mutability of self in Cheval::render()
- [ ] Add animations
- [ ] Add audio support
- [ ] Check file watcher (seems to be broken sometimes)
- [ ] Allow elements to register for http (or just sign them up all)
- [ ] Allow using css names for colors

## Obsolete
~ - [#] Refactor Variable to allow baking of value ~


## DONE
- [x] Add BakedExpression
- [x] Add bounding box support for text rendering
- [x] Add timer element
- [x] Add http API (setVariable)
- [x] Implement font cache
- [x] Text rendering
- [x] Add clean error handling
- [x] Render to png
