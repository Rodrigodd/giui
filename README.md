# fix

- The events being a ```Box<dyn Any>``` and the functions receiving a ```<T: 'static>``` which is converted to 
  ```Box<dyn Any>``` is confusing... Sometimes I try pass a Box<dyn Any> to the function, and any error happen

# Todo

- Add animation support
- Implement more widgets:
  - ~~Button~~
  - ~~Slider~~
  - ~~Toggle~~
  - Dropbox
  - ~~Hover~~
  - Fold
  - Text Field
  - ~~Scrollbar~~
- Add some richtext support?
- Make ever rect masks its content?