# Giui

A normal GUI crate made by myself.

# Features

- Render agnostic:
  - everything is render to textured axis aligned rects; loading the textures
    and rendering them is up to you.
- Event Loop agnostic:
  - There are some functions like `Gui::handle_events()`,
    `Gui::handle_scheduled_events()`, `Gui::render_is_dirty()` and
    `Gui::cursor_change()`; calling them in the right place is up to you.
  - Currently the expose API only accept Winit events, but I should expose the
    underline event API, and allow you to translate the events from your event
    loop, so you would be able to replace Winit with SDL for example.
- Highly customizable
  - this means that there is no default graphics
- No advanced GUI architectures
  - widgets use callback closures, but only the widgets that I made, not in the
    core of crate at least. I expect to be able to make widgets that use others
    event systems.
  - there are no reactive events, or something like that, controls need to be
    updated manually.
  - I expected that I could build that type of things over the existing system.
- Automatic layout, based on Godot layout system:
  - each layout compute its own min_size based on its children's min_size
  - each layout sets its children's rect, after its own rect is set by its
    parent
- Rich Text Layout, or so I plan
  - [x] Basic shaping. There is also harfbuzz backend, but my naive rushed
  implementation is taking multiples ms to shape a single screen full of text.
  - [x] Style spans of text: for now you can define spans of text that have a
  different color, font or font size; a selection highlight; or a colored
  underline.
  - [ ] font fallback
  - [ ] query system fonts (or maybe only a generic interface for it)

- Features that I expect to add some day:
  - [ ] multi window support (you can create multiple Gui's, one for each window,
  but that don't really work)
    - [ ] support for modal windows and popups (this may only depend in the Winit
    side of things, after adding the multi window support)
  - [ ] animations: there is a AnimatedIcon, but I want to add a way of
  registering animations, that would be basically closures that will be called
  every frame. This would be enough to add animated layout transitions?
  - [ ] scripting support?
