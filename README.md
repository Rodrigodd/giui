# fix

- The events being a ```Box<dyn Any>``` and the functions receiving a ```<T: 'static>``` which is converted to 
  ```Box<dyn Any>``` is confusing... Sometimes I try pass a Box<dyn Any> to the function, and any error happen

# Todo

- Add animation support
- Implement more widgets:
  - ~~Button~~
  - ~~Slider~~
  - ~~Toggle~~
  - ~~Dropdown~~
  - ~~Hover~~
  - Fold
  - Text Field
  - ~~Scrollbar~~
- Add some richtext support?
- Replace each 'aling: i8' with a enum