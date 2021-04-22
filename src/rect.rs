use crate::util::cmp_float;

#[derive(Copy, Clone, Debug)]
pub enum RectFill {
    Fill,
    ShrinkStart,
    ShrinkCenter,
    ShrinkEnd,
}
impl Default for RectFill {
    fn default() -> Self {
        RectFill::Fill
    }
}

bitflags! {
    pub struct RenderDirtyFlags: u8 {
        /// The width of the rect has changed
        const WIDTH = 0x1;
        /// The height of the rect has changed
        const HEIGHT = 0x2;
        /// The rect of the rect has changed
        const RECT = 0x4;
    }
}
impl Default for RenderDirtyFlags {
    fn default() -> Self {
        RenderDirtyFlags::all()
    }
}

bitflags! {
    pub struct LayoutDirtyFlags: u16 {
        /// The width of the rect has changed
        const WIDTH = 0x01;
        /// The height of the rect has changed
        const HEIGHT = 0x02;
        /// The rect of the rect has changed
        const RECT = 0x04;

        const MIN_WIDTH = 0x08;
        const MIN_HEIGHT = 0x10;

        const DIRTY = 0x20;
    }
}
impl Default for LayoutDirtyFlags {
    fn default() -> Self {
        LayoutDirtyFlags::all()
    }
}

/// The basic component of a UI element.
/// The final rect of the element is calculate
/// by the fomula ```anchor*parend_size + margins```.
///
/// For example, a Rect with margins 0 0 0 0, and
/// margins 10 10 40 40, will be a rect always located
/// in the top left corner of the screen, in the position
/// 10 10, and with width and height 30 30.
pub struct Rect {
    pub anchors: [f32; 4],
    pub margins: [f32; 4],
    pub(crate) user_min_size: [f32; 2],
    pub(crate) min_size: [f32; 2],
    pub(crate) rect: [f32; 4],
    pub(crate) expand_x: bool,
    pub(crate) expand_y: bool,
    pub(crate) fill_x: RectFill,
    pub(crate) fill_y: RectFill,
    pub ratio_x: f32,
    pub ratio_y: f32,
    pub(crate) render_dirty_flags: RenderDirtyFlags,
    pub(crate) layout_dirty_flags: LayoutDirtyFlags,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            anchors: [0.0, 0.0, 1.0, 1.0],
            margins: [0.0, 0.0, 0.0, 0.0],
            user_min_size: [0.0; 2],
            min_size: [0.0; 2],
            rect: [0.0; 4],
            expand_x: false,
            expand_y: false,
            fill_x: RectFill::default(),
            fill_y: RectFill::default(),
            ratio_x: 1.0,
            ratio_y: 1.0,
            render_dirty_flags: RenderDirtyFlags::default(),
            layout_dirty_flags: LayoutDirtyFlags::default(),
        }
    }
}
impl Rect {
    pub fn new(anchors: [f32; 4], margins: [f32; 4]) -> Self {
        Self {
            anchors,
            margins,
            ..Default::default()
        }
    }

    /// Get the dirty flags. The dirty flags keep track if some values have changed
    /// since last call to clear_dirty_flags.
    pub fn get_render_dirty_flags(&mut self) -> RenderDirtyFlags {
        self.render_dirty_flags
    }

    pub fn clear_render_dirty_flags(&mut self) {
        self.render_dirty_flags = RenderDirtyFlags::empty();
    }

    pub fn dirty_render_dirty_flags(&mut self) {
        self.render_dirty_flags = RenderDirtyFlags::all();
    }

    pub fn get_layout_dirty_flags(&mut self) -> LayoutDirtyFlags {
        self.layout_dirty_flags
    }

    pub fn clear_layout_dirty_flags(&mut self) {
        self.layout_dirty_flags = LayoutDirtyFlags::empty();
    }

    pub fn dirty_layout_dirty_flags(&mut self) {
        self.layout_dirty_flags = LayoutDirtyFlags::all();
    }

    pub fn set_rect(&mut self, rect: [f32; 4]) {
        #[allow(clippy::float_cmp)]
        if rect == self.rect {
            return;
        }
        self.render_dirty_flags.insert(RenderDirtyFlags::RECT);
        self.layout_dirty_flags.insert(LayoutDirtyFlags::RECT);
        if !cmp_float(self.get_width(), rect[2] - rect[0]) {
            self.render_dirty_flags.insert(RenderDirtyFlags::WIDTH);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::WIDTH);
        }
        if !cmp_float(self.get_height(), rect[3] - rect[1]) {
            self.render_dirty_flags.insert(RenderDirtyFlags::HEIGHT);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::HEIGHT);
        }
        self.rect = rect;
    }

    /// Set the designed area for this rect. This rect will decide its own size,
    /// based on its size flags and the designed area.
    pub fn set_designed_rect(&mut self, rect: [f32; 4]) {
        let mut new_rect = [0.0; 4];
        if rect[2] - rect[0] <= self.get_min_size()[0] {
            new_rect[0] = rect[0];
            new_rect[2] = rect[0] + self.get_min_size()[0];
        } else {
            match self.fill_x {
                RectFill::Fill => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[2];
                }
                RectFill::ShrinkStart => {
                    new_rect[0] = rect[0];
                    new_rect[2] = rect[0] + self.get_min_size()[0];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[2] - rect[0] - self.get_min_size()[0]) / 2.0;
                    new_rect[0] = rect[0] + x;
                    new_rect[2] = rect[2] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[0] = rect[2] - self.get_min_size()[0];
                    new_rect[2] = rect[2];
                }
            }
        }

        if rect[3] - rect[1] <= self.get_min_size()[1] {
            new_rect[1] = rect[1];
            new_rect[3] = rect[1] + self.get_min_size()[1];
        } else {
            match self.fill_y {
                RectFill::Fill => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[3];
                }
                RectFill::ShrinkStart => {
                    new_rect[1] = rect[1];
                    new_rect[3] = rect[1] + self.get_min_size()[1];
                }
                RectFill::ShrinkCenter => {
                    let x = (rect[3] - rect[1] - self.get_min_size()[1]) / 2.0;
                    new_rect[1] = rect[1] + x;
                    new_rect[3] = rect[3] - x;
                }
                RectFill::ShrinkEnd => {
                    new_rect[1] = rect[3] - self.get_min_size()[1];
                    new_rect[3] = rect[3];
                }
            }
        }
        self.set_rect(new_rect);
    }

    pub fn set_fill_x(&mut self, fill: RectFill) {
        self.fill_x = fill;
    }

    pub fn set_fill_y(&mut self, fill: RectFill) {
        self.fill_y = fill;
    }

    #[inline]
    pub fn get_min_size(&self) -> [f32; 2] {
        self.min_size
    }

    #[inline]
    pub fn set_min_size(&mut self, min_size: [f32; 2]) {
        self.user_min_size = min_size;
        let min_size = [
            self.min_size[0].max(min_size[0]),
            self.min_size[1].max(min_size[1]),
        ];

        if !cmp_float(self.min_size[0], min_size[0]) {
            self.layout_dirty_flags.insert(LayoutDirtyFlags::MIN_WIDTH);
            self.min_size[0] = min_size[0];
        }
        if !cmp_float(self.min_size[1], min_size[1]) {
            self.layout_dirty_flags.insert(LayoutDirtyFlags::MIN_HEIGHT);
            self.min_size[1] = min_size[1];
        }

        if self.get_width() < self.get_min_size()[0] {
            self.set_width(min_size[0]);
        }
        if self.get_height() < self.get_min_size()[1] {
            self.set_height(min_size[1]);
        }
    }

    /// Return true if this have the size_flag::EXPAND_X flag.
    #[inline]
    pub fn is_expand_x(&self) -> bool {
        self.expand_x
    }

    /// Return true if this have the size_flag::EXPAND_Y flag.
    #[inline]
    pub fn is_expand_y(&self) -> bool {
        self.expand_y
    }

    #[inline]
    pub fn get_top_left(&self) -> (f32, f32) {
        (self.rect[0], self.rect[1])
    }

    #[inline]
    pub fn get_rect(&self) -> &[f32; 4] {
        &self.rect
    }

    #[inline]
    pub fn get_center(&self) -> (f32, f32) {
        (
            (self.rect[0] + self.rect[2]) / 2.0,
            (self.rect[1] + self.rect[3]) / 2.0,
        )
    }

    #[inline]
    pub fn get_width(&self) -> f32 {
        self.rect[2] - self.rect[0]
    }

    #[inline]
    pub fn set_width(&mut self, width: f32) {
        if !cmp_float(self.get_width(), width) {
            self.render_dirty_flags.insert(RenderDirtyFlags::WIDTH);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::WIDTH);
        }
        self.rect[2] = self.rect[0] + width;
    }

    #[inline]
    pub fn get_height(&self) -> f32 {
        self.rect[3] - self.rect[1]
    }

    #[inline]
    pub fn set_height(&mut self, height: f32) {
        if !cmp_float(self.get_height(), height) {
            self.render_dirty_flags.insert(RenderDirtyFlags::HEIGHT);
            self.layout_dirty_flags.insert(LayoutDirtyFlags::HEIGHT);
        }
        self.rect[3] = self.rect[1] + height;
    }

    #[inline]
    pub fn get_size(&self) -> [f32; 2] {
        [self.rect[2] - self.rect[0], self.rect[3] - self.rect[1]]
    }

    #[inline]
    pub fn get_relative_x(&self, x: f32) -> f32 {
        (x - self.rect[0]) / self.get_width()
    }

    #[inline]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect[0] < x && x < self.rect[2] && self.rect[1] < y && y < self.rect[3]
    }
}
