# Todo

- Add animation support
- Add layout support
- Implement more widgets:
  - ~~Button~~
  - ~~Slider~~
  - ~~Toggle~~
  - Dropbox
  - Hover // more than one behaviour per widget?
  - Fold
  - Text Field
  - Scrollbar?
- Implement a
- Add some richtext support?


```
painel = Painel(texture, [0.0, 0.0, 1.0, 1.0], 5.0)
menu = {
  rect = Rect([0.0, 0.0, 0.0, 1.0], [10.0, 10.0, 190.0, -10.0])
  gui.add_widget(Widget(rect, painel, None), None)
}
right_painel = {
  rect = Rect([0.0, 0.0, 1.0, 1.0], [200.0, 10.0, -10.0, -10.0])
  gui.add_widget(Widget(rect, painel, None), None)
}
top_text = {
  text_box =  Widget(
      Rect([0.0, 0.0, 1.0, 0.5], [15.0, 15.0, -15.0, -7.5]),
      painel.with_color(#c8c8c8ff),
      None,
      right_painel,
    )
  graphic = gui.render().add_text(Text(
    "This is a example text. Please, don't mind me. Continue doing what you need to do. If you cannot ignore this text, I don't mind.".to_string(),
    20.0,
    (0, -1),
  ))
  gui.add_widget(
    Widget(
      Rect([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
      graphic.clone(),
      None,
    ),
    text_box,
  )
  graphic
}
bottom_text = {
  graphic = painel.with_color(#c8c8c8ff),
  text_box = Widget(
    Rect([0.0, 0.5, 1.0, 1.0], [15.0, 7.5, -15.0, -15.0]),
    graphic,
    None,
    right_painel,
  ),
  graphic = Text(
    "This is another example text. Please, also don't mind me. Continue doing what you was doing. If you cannot ignore this text, I don't mind either.".to_string(),
    20.0,
    (-1, 0),
  )
  Widget(
    Rect([0.0, 0.0, 1.0, 1.0], [5.0, 5.0, -5.0, -5.0]),
    graphic,
    None,
    text_box,
  ),
  text_box
}

my_button = {
  graphic = painel.with_color(#c8c8c8ff)
  button = Widget(
    Rect([0.0, 0.0, 1.0, 0.0], [5.0, 5.0, -5.0, 35.0]),
    graphic,
    Button(),
    menu,
  ),
  graphic = Text("My Button".to_string(), 16.0, (0, 0)).with_color(#282828ff),
  Widget(
    Rect([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]),
    graphic,
    None,
    button,
  ),
  button
}
my_slider = {
  slider = {
    graphic = None
    Widget(
      Rect([0.0, 0.0, 1.0, 0.0], [5.0, 40.0, -5.0, 75.0]),
      graphic,
      None,
      menu,
    ),
  }
  slide_area = {
    graphic = painel.with_color([170, 170, 170, 255]),
    Widget(
      Rect([0.0, 0.5, 1.0, 0.5], [10.0, -3.0, -10.0, 3.0]),
      graphic,
      None,
      slider,
    ),
  }
  handle = {
    graphic = painel.with_color(#c8c8c8ff)
    Widget(
      Rect([0.5, 0.5, 0.5, 0.5], [-3.0, -14.0, 3.0, 14.0]),
      graphic,
      None,
      slide_area,
    ),
  }
  gui.set_behaviour_of(
    slider,
    Box(Slider(handle, slide_area, 10.0, 30.0, 25.0))),
  slider
}
my_toggle = {
  toggle = Widget(
    Rect([0.0, 0.0, 1.0, 0.0], [5.0, 80.0, -5.0, 115.0]),
    None,
    None,
    menu,
  ),

  background = {
    graphic = painel
      .with_color(#c8c8c8ff)
      .with_border(0.0)
    Widget(
      Rect([0.0, 0.5, 0.0, 0.5], [5.0, -10.0, 25.0, 10.0]),
      graphic,
      None,
      toggle,
    )
  }
  marker = {
    graphic = painel
      .clone()
      .with_color(#000000ff)
      .with_border(0.0),
    Widget(
      Rect([0.5, 0.5, 0.5, 0.5], [-6.0, -6.0, 6.0, 6.0]),
      graphic,
      None,
      background,
    ),
  }
  gui.set_behaviour_of(toggle, Box(Toggle(background, marker))))

  {
    graphic = Text("Bottom Text".to_string(), 16.0, (-1, 0))
      .with_color([40, 40, 100, 255]),
    Widget(
      Rect([0.0, 0.0, 1.0, 1.0], [30.0, 0.0, 0.0, 0.0]),
      graphic,
      None,
      toggle,
    ),
  }
  toggle
}
```