TODO: This file is not up to date. Sorry.


## In Progress


- [ ] Build packages automatically after push to github.

- [ ] Use tracing for all debugging
	- [x] Enable tracing, and start using it

		
## TODO

- [ ] Remember previous window positions for windows that are not currently used
- [ ] Allow writing of png sequences for window mode png
- [ ] {verify} Fix alpha handling

### post v0.2

- [ ] Combine RGB565 and RGBA8888 mode for fbdev backend
- [ ] Remove actix, and use axum instead

- [ ] Put profiling behind feature flag
- [ ] Put profiling behind command line flag

- [ ] Fix mutability of self in Cheval::render()

### post v0.3

- [ ] Extract shared rendering functions -> RenderBuffer package
- [ ] Replace :HACK: text effects with better version

- [ ] Improve reporting of broken config files

- [ ] Refactor FileCache::run

- [ ] Remove unneeded copying from FileCache aka buffer bloat
- [ ] Double check why outer FileCache needs to be Arc< Mutex< ... > >

- [ ] Add sound effect support
	- [ ] Fix time expire sound to be more precise

- [ ] Add event system
- [ ] Soundbank element should own it's soundbank, and react to events
- [ ] Extract ElementConfig parameters

- [ ] Make FileCache watcher async
- [ ] Optimise pathes to watch for FileCache notify

- [ ] Use BakedExpression where sensible
	- [x] Use BakedExpression for text position, and bounding box config
	- [x] Use BakedExpression for text
	- [ ] Verify we use BakedExpressions everywhere

- [ ] Fix missing initialization of active page for multi page configs

- [ ] Cleanup config file selection, and error reporting
- [ ] Make Windows work
- [ ] Persist variables in regular intervals
- [ ] Enable debug frames via command line 
- [ ] Merge goto next&prev page into turn page with direction
- [ ] Decide on http response for page changes
- [ ] Fix default for bounding boxes
- [ ] Change config format
	- [ ] Add general config section
	- [x] Add support for multiple "pages"
	- [x] Add support for default values for variables
- [ ] Add more position modes, e.g negative values for "from bottom/right"
- [ ] Add clipping at buffer edges
	- [x] Add clipping for text
- [ ] Auto reloading on config change
- [ ] Make config loading more robust
- [ ] Improve Debug and/or Display traits for elements
- [ ] Allow parameters for windows
- [ ] Add animations
	- [ ] Use spline/bezier based files for values
- [ ] Add audio support -> is started
- [ ] Allow elements to register for http (or just sign them up all)
- [ ] Handle strings with whitespace in expressions

## Obsolete

~ - [#] Refactor Variable to allow baking of value ~


## DONE

- [x] Enable profiling via feature
- [x] Update packages
- [x] Merge, and kill `multi-window` branch for good.
- [x] Fix crash when moving right without pages
- [x] Upgrade edition to 2021
- [x] Allow window title prefix to be passed on command line
- [x] Add file watcher to FileCache to allow automatic updates
	- [x] Poll files for changes
	- [x] or: Use operating system to get notified about changes

- [x] Use file cache for image element, including hot reload
- [x] Replace/refactor LoadTextElement -> delete?
	- [x] Removed ~Check~ file watcher (seems to be broken sometimes) (merge with FileCache?!)
- [x] Add multi line support to TextElement
- [x] Add text_lines_from_file to expressions to allow loading specific lines from file
- [x] Add text_from_file to expressions to allow loading text from file
- [x] Rename `master` branch to `main`
- [x] Merge `multi-window` branch
- [x] Add text effects (shadow, glow?) to text element
	- [x] Add text shadow
	- [x] Add text glow (:HACK:)
- [x] Send actual resulting value back when changing variables via http
- [x] Allow selecting of variable, and inc/dec of that selected variable
- [x] Fix set/inc/dec variable api to respond with resulting value (and name)
- [x] Allow incrementing & decrementing variable via http api
- [x] Fix alpha blending for blocks
- [x] Allow using css names for colors
- [x] Default to current directory for configuration
- [x] Use \*config.yaml if exactly one matches
- [x] Check for config.yaml if config parameter is a directory
- [x] Make filepath in config file relative to config file
- [x] Allow keyboard input from console
- [x] Add support for animated images
- [x] Add visual countdown example
- [x] Fix http interface to allow setting of f32
- [x] Add support for default variable values from 
- [x] Add BakedExpression
- [x] Add bounding box support for text rendering
- [x] Add timer element
- [x] Add http API (setVariable)
- [x] Implement font cache
- [x] Text rendering
- [x] Add clean error handling
- [x] Render to png
