use crate::{
    util::cmp_float, widgets::SetScrollPosition, Behaviour, Context, ControlBuilder, Id,
    InputFlags, KeyboardEvent, Layout, LayoutContext, MinSizeContext,
};

use std::{any::Any, collections::BTreeMap};
use winit::event::VirtualKeyCode;

pub struct SetList<T>(pub Vec<T>);

#[derive(Default)]
pub struct ListViewLayout {
    h: bool,
    v: bool,
}
impl ListViewLayout {
    pub fn new(h: bool, v: bool) -> Self {
        Self { h, v }
    }
}
impl Layout for ListViewLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        let content = match ctx.get_children(this).get(0) {
            Some(x) => *x,
            None => return [0.0; 2],
        };
        let mut min_size = [0.0, 0.0];
        let content_min_size = ctx.get_min_size(content);
        if !self.h {
            min_size[0] = content_min_size[0];
        }
        if !self.v {
            min_size[1] = content_min_size[1];
        }

        min_size
    }

    fn update_layouts(&mut self, _this: Id, _ctx: &mut LayoutContext) {}
}

#[derive(Debug, Clone)]
struct CreatedItem {
    id: Id,
    i: usize,
    // top position relative to the top of the view, when created
    y: f32,
    height: f32,
}
impl CreatedItem {
    fn new(id: Id, i: usize, y: f32, height: f32) -> Self {
        Self { id, i, y, height }
    }
}

pub struct List<T: 'static, F: for<'a> FnMut(&T, Id, ControlBuilder<'a>) -> ControlBuilder<'a>> {
    spacing: f32,
    margins: [f32; 4],
    content_width: f32,
    delta_x: f32,
    delta_y: f32,
    last_delta_x: f32,
    set_y: Option<f32>,
    start_y: f32,
    end_y: f32,
    last_rect: [f32; 4],
    view: Id,
    v_scroll_bar: Id,
    v_scroll_bar_handle: Id,
    h_scroll_bar: Id,
    h_scroll_bar_handle: Id,
    items: Vec<T>,
    last_created_items: BTreeMap<usize, CreatedItem>,
    created_items: BTreeMap<usize, CreatedItem>,
    focused: Option<CreatedItem>,
    create_item: F,
}
impl<T: 'static, F: for<'a> FnMut(&T, Id, ControlBuilder<'a>) -> ControlBuilder<'a>> List<T, F> {
    /// v_scroll must be a descendant of this
    pub fn new(
        content_width: f32,
        view: Id,
        v_scroll_bar: Id,
        v_scroll_bar_handle: Id,
        h_scroll_bar: Id,
        h_scroll_bar_handle: Id,
        items: Vec<T>,
        create_item: F,
    ) -> Self {
        Self {
            // TODO: spacing and margins must be paramenters
            spacing: 2.0,
            margins: [2.0; 4],
            content_width,
            delta_x: 0.0,
            delta_y: 0.0,
            last_delta_x: f32::NAN,
            set_y: Some(0.0),
            start_y: 0.0,
            end_y: 0.0,
            last_rect: [0.0; 4],
            view,
            v_scroll_bar,
            v_scroll_bar_handle,
            h_scroll_bar,
            h_scroll_bar_handle,
            items,
            focused: None,
            last_created_items: BTreeMap::new(),
            created_items: BTreeMap::new(),
            create_item,
        }
    }

    fn create_item(
        &mut self,
        i: usize,
        this: Id,
        y: f32,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        if self.focused.as_ref().map_or(false, |x| x.i == i) {
            let x = self.focused.as_mut().unwrap();
            let id = x.id;
            println!("move focused {}", id);
            let height = ctx.get_min_size(id)[1];
            ctx.set_designed_rect(
                id,
                [
                    view_rect[0] - self.delta_x,
                    y,
                    view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                    y + height,
                ],
            );
            ctx.move_to_front(id);
            x.y = y - view_rect[1];
            x.height = height;
            self.created_items.insert(i, x.clone());
            self.last_created_items.remove(&i);
            return height;
        }

        match self.last_created_items.remove(&i) {
            Some(mut x) => {
                let id = x.id;
                println!("move {}", id);
                let height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                ctx.move_to_front(id);
                x.y = y - view_rect[1];
                x.height = height;
                self.created_items.insert(i, x);
                height
            }
            None => {
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                let height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                self.created_items
                    .insert(i, CreatedItem::new(id, i, y - view_rect[1], height));
                height
            }
        }
    }

    fn create_item_at(
        &mut self,
        start_y: f32,
        this: Id,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        let i = start_y as usize;

        if self.focused.as_ref().map_or(false, |x| x.i == i) {
            let x = self.focused.as_mut().unwrap();
            let id = x.id;
            println!("move focused {}", id);
            let height = ctx.get_min_size(id)[1];
            let mut y = view_rect[1] - start_y.fract() * (height + self.spacing);
            ctx.set_designed_rect(
                id,
                [
                    view_rect[0] - self.delta_x,
                    y,
                    view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                    y + height,
                ],
            );
            ctx.move_to_front(id);
            x.y = y - view_rect[1];
            x.height = height;
            y += height;
            self.created_items.insert(i, x.clone());
            self.last_created_items.remove(&i);
            return y;
        }
        
        match self.last_created_items.remove(&i) {
            Some(mut x) => {
                let id = x.id;
                println!("move {}", id);
                let height = ctx.get_min_size(id)[1];
                let mut y = view_rect[1] - start_y.fract() * (height + self.spacing);
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                ctx.move_to_front(id);
                x.y = y - view_rect[1];
                x.height = height;
                y += height;
                self.created_items.insert(i, x);
                y
            }
            None => {
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                let height = ctx.get_min_size(id)[1];
                let mut y = view_rect[1] - start_y.fract() * (height + self.spacing);
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                self.created_items
                    .insert(i, CreatedItem::new(id, i, y - view_rect[1], height));
                y += height;
                y
            }
        }
    }

    fn create_item_from_bottom(
        &mut self,
        i: usize,
        this: Id,
        y: f32,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        if self.focused.as_ref().map_or(false, |x| x.i == i) {
            let x = self.focused.as_mut().unwrap();
            let id = x.id;
            println!("move focused {}", id);
            let height = ctx.get_min_size(id)[1];
            ctx.set_designed_rect(
                id,
                [
                    view_rect[0] - self.delta_x,
                    y - height,
                    view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                    y,
                ],
            );
            ctx.move_to_back(id);
            x.y = y - height - view_rect[1];
            x.height = height;
            self.created_items.insert(i, x.clone());
            self.last_created_items.remove(&i);
            return height;
        }

        match self.last_created_items.remove(&i) {
            Some(mut x) => {
                let id = x.id;
                println!("move {}", id);
                let height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y - height,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y,
                    ],
                );
                ctx.move_to_back(id);
                x.y = y - height - view_rect[1];
                x.height = height;
                self.created_items.insert(i, x);
                height
            }
            None => {
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                let height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y - height,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y,
                    ],
                );
                ctx.move_to_back(id);
                self.created_items.insert(
                    i,
                    CreatedItem::new(id, i, y - height - view_rect[1], height),
                );
                height
            }
        }
    }

    fn create_items_from_top(&mut self, view_rect: [f32; 4], this: Id, ctx: &mut LayoutContext) {
        println!("create from top!");

        self.last_created_items.append(&mut self.created_items);

        let mut i = 0;
        self.start_y = 0.0;
        self.delta_y = 0.0;

        let mut stop = usize::max_value();
        let mut height = 0.0;
        let mut y = view_rect[1];

        // create items below, if necessary
        while y < view_rect[3] {
            // there is not enough items to fill the view
            if i >= self.items.len() {
                self.end_y = self.items.len() as f32;
                println!("end at {}", self.end_y);
                return;
            }
            height = self.create_item(i, this, y, ctx, view_rect);
            y += self.spacing + height;
            i += 1;
        }

        self.end_y =
            (i - 1) as f32 + (view_rect[3] - (y - height - self.spacing)) / (height + self.spacing);
        println!("end at {}", self.end_y);
    }

    fn create_items_from_bottom(&mut self, view_rect: [f32; 4], this: Id, ctx: &mut LayoutContext) {
        self.last_rect = view_rect;
        self.end_y = self.items.len() as f32;

        println!("create items from_bottom");

        self.last_created_items.append(&mut self.created_items);

        let mut i = self.items.len() - 1;
        let mut y = view_rect[3];

        while y > view_rect[1] {
            let height = self.create_item_from_bottom(i, this, y, ctx, view_rect);
            y -= height + self.spacing;
            // if the top item don't fill the view yet, create the items from top
            if i == 0 {
                if y > view_rect[1] {
                    return self.create_items_from_top(view_rect, this, ctx);
                }
                break;
            }
            i -= 1;
        }

        {
            let start_control = self.created_items.iter().next().unwrap().1;
            let id = start_control.id;
            let height = ctx.get_min_size(id)[1];
            let i = start_control.i;
            let y = start_control.y + view_rect[1];

            self.start_y = (i as f32) + (view_rect[1] - y) / (height + self.spacing);
            dbg!(self.start_y);
            dbg!(i);
            println!("start_y: {}", self.start_y);
        }
    }

    // TODO: make this reuse already created items
    fn create_items_from_a_start_y(
        &mut self,
        start_y: f32,
        view_rect: [f32; 4],
        this: Id,
        ctx: &mut LayoutContext,
    ) {
        if cmp_float(start_y, 0.0) {
            return self.create_items_from_top(view_rect, this, ctx);
        }
        println!("create from zero!");

        self.start_y = start_y;
        let mut i = self.start_y as usize;

        if i >= self.items.len() {
            return self.create_items_from_bottom(view_rect, this, ctx);
        }

        let mut y = self.create_item_at(start_y, this, ctx, view_rect);
        y += self.spacing;
        i += 1;

        if i >= self.items.len() && y - self.spacing < view_rect[3] {
            return self.create_items_from_bottom(view_rect, this, ctx);
        }

        while y < view_rect[3] {
            let height = self.create_item(i, this, y, ctx, view_rect);
            y += self.spacing + height;
            i += 1;
            if i >= self.items.len() {
                if y - self.spacing < view_rect[3] {
                    return self.create_items_from_bottom(view_rect, this, ctx);
                }
                break;
            }
        }

        {
            let last = self.created_items.iter().rev().next().unwrap().1;
            let id = last.id;
            let height = ctx.get_min_size(id)[1];
            let i = last.i;
            let y = view_rect[1] + last.y;

            self.end_y = i as f32 + (view_rect[3] - y) / (height + self.spacing);
        }
        println!("end at {}", self.end_y);
    }

    fn create_items(&mut self, view_rect: [f32; 4], this: Id, ctx: &mut LayoutContext) {
        let same_rect = cmp_float(view_rect[0], self.last_rect[0])
            && cmp_float(view_rect[1], self.last_rect[1])
            && cmp_float(view_rect[2], self.last_rect[2])
            && cmp_float(view_rect[3], self.last_rect[3]);
        println!("delta_y: {}", self.delta_y);
        if same_rect
            && cmp_float(0.0, self.delta_y)
            && self.set_y.is_none()
            && cmp_float(self.last_delta_x, self.delta_x)
            && !self.items.is_empty()
        {
            return;
        }

        std::mem::swap(&mut self.created_items, &mut self.last_created_items);
        debug_assert!(self.created_items.is_empty());

        self.last_rect = view_rect;
        self.last_delta_x = self.delta_x;
        let delta_y = self.delta_y;
        self.delta_y = 0.0;

        if self.last_created_items.is_empty() {
            self.create_items_from_top(view_rect, this, ctx);
        } else if let Some(y) = self.set_y.take() {
            self.create_items_from_a_start_y(y, view_rect, this, ctx);
        } else {
            if delta_y < 0.0 {
                // create items above
                let mut i = self.start_y as usize;
                let start_control = match self.last_created_items.get(&i) {
                    Some(x) => x,
                    None => {
                        dbg!(self.start_y);
                        dbg!(i);
                        dbg!(&self.last_created_items);
                        panic!("ahhhhh!!");
                    }
                };
                let mut y = start_control.y + view_rect[1] - delta_y;
                while y > view_rect[1] {
                    if i == 0 {
                        return self.create_items_from_top(view_rect, this, ctx);
                    }
                    y -= self.spacing;
                    i -= 1;
                    let height = self.create_item_from_bottom(i, this, y, ctx, view_rect);
                    y -= height;
                }
            }

            let mut i = self.start_y as usize;
            let start_control = self.last_created_items.get(&i).unwrap();
            let mut y = start_control.y + view_rect[1] - delta_y;

            if i >= self.items.len() && y - self.spacing < view_rect[3] {
                return self.create_items_from_bottom(view_rect, this, ctx);
            }

            // create items below, if necessary
            while y < view_rect[3] {
                let height = self.create_item(i, this, y, ctx, view_rect);
                y += self.spacing + height;
                i += 1;
                if i >= self.items.len() {
                    if y - self.spacing < view_rect[3] {
                        return self.create_items_from_bottom(view_rect, this, ctx);
                    }
                    break;
                }
            }

            {
                let start_control = self.created_items.iter().next().unwrap().1;
                let id = start_control.id;
                let height = ctx.get_min_size(id)[1];
                let i = start_control.i;
                let y = start_control.y + view_rect[1];

                self.start_y = i as f32 + (view_rect[1] - y) / (height + self.spacing);
            }
            {
                let last = self.created_items.iter().rev().next().unwrap().1;
                let id = last.id;
                let height = ctx.get_min_size(id)[1];
                let i = last.i;
                let y = view_rect[1] + last.y;

                self.end_y = i as f32 + (view_rect[3] - y) / (height + self.spacing);
            }
            println!("start at {}", self.start_y);
            println!("end at {}", self.end_y);
        }
    }
}
impl<T: 'static, F: for<'a> FnMut(&T, Id, ControlBuilder<'a>) -> ControlBuilder<'a>> Behaviour
    for List<T, F>
{
    fn on_start(&mut self, _this: Id, ctx: &mut Context) {
        ctx.move_to_front(self.h_scroll_bar);
        ctx.move_to_front(self.v_scroll_bar);
    }

    fn on_focus_change(&mut self, focus: bool, _this: Id, ctx: &mut Context) {
        if focus {
            self.focused = self
                .created_items
                .iter()
                .find(|(_, x)| ctx.is_focus(x.id))
                .map(|(_, x)| x)
                .cloned();
        } else {
            self.focused = None;
        }
        dbg!(&self.focused);
    }

    fn on_active(&mut self, _this: Id, ctx: &mut Context) {
        let view_rect = ctx.get_rect(self.view);

        let view_width = view_rect[2] - view_rect[0];

        ctx.set_anchor_left(self.h_scroll_bar_handle, self.delta_x / self.content_width);
        ctx.set_anchor_right(
            self.h_scroll_bar_handle,
            ((self.delta_x + view_width) / self.content_width).min(1.0),
        );

        ctx.set_anchor_top(
            self.v_scroll_bar_handle,
            self.start_y / self.items.len() as f32,
        );
        ctx.set_anchor_bottom(
            self.v_scroll_bar_handle,
            (self.end_y / self.items.len() as f32).min(1.0),
        );
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(event) = event.downcast_ref::<SetScrollPosition>() {
            if !event.vertical {
                let total_size = self.content_width - ctx.get_size(self.view)[0];
                self.delta_x = event.value.max(0.0) * total_size;
            } else {
                let total_size = self.items.len() as f32 - (self.end_y - self.start_y);
                self.set_y = Some(event.value.max(0.0) * total_size);
                println!("set y to {:?}", self.set_y);
            }
            ctx.dirty_layout(self.view);
            ctx.dirty_layout(this);
        } else if event.is::<SetList<T>>() {
            self.items = event.downcast::<SetList<T>>().unwrap().0;
            self.set_y = Some(0.0);
            for (_, x) in self.created_items.iter() {
                println!("remove {}", x.id);
                ctx.remove(x.id);
            }
            self.focused = None;
            self.created_items.clear();
            ctx.dirty_layout(this);
        }
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE | InputFlags::SCROLL
    }

    fn on_scroll_event(&mut self, delta: [f32; 2], _this: Id, ctx: &mut Context) {
        if !cmp_float(delta[0], 0.0) {
            self.delta_x += delta[0];
            ctx.dirty_layout(self.view);
        }

        // if items are all displayed, there is no need for vertical scroll
        if cmp_float(self.start_y, 0.0) && cmp_float(self.end_y, self.items.len() as f32) {
            return;
        }

        if !cmp_float(delta[1], 0.0) {
            self.delta_y -= delta[1];
            ctx.dirty_layout(self.view);
        }
    }

    fn on_keyboard_event(&mut self, event: KeyboardEvent, _this: Id, ctx: &mut Context) -> bool {
        match event {
            KeyboardEvent::Pressed(key) => match key {
                VirtualKeyCode::Up => {
                    self.delta_y -= 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Down => {
                    self.delta_y += 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Right => {
                    self.delta_x += 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Left => {
                    self.delta_x -= 30.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::Home => {
                    self.delta_y = 0.0;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::End => {
                    self.delta_y = f32::INFINITY;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::PageUp => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y -= height;
                    ctx.dirty_layout(self.view);
                    true
                }
                VirtualKeyCode::PageDown => {
                    let height = ctx.get_size(self.view)[1] - 40.0;
                    self.delta_y += height;
                    ctx.dirty_layout(self.view);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
impl<T: 'static, F: for<'a> FnMut(&T, Id, ControlBuilder<'a>) -> ControlBuilder<'a>> Layout
    for List<T, F>
{
    fn compute_min_size(&mut self, _this: Id, ctx: &mut MinSizeContext) -> [f32; 2] {
        let mut min_size = ctx.get_min_size(self.view);

        let h_scroll_bar_size = ctx.get_min_size(self.h_scroll_bar);
        let v_scroll_bar_size = ctx.get_min_size(self.v_scroll_bar);

        min_size[0] = min_size[0].max(h_scroll_bar_size[0]);
        min_size[1] = min_size[1].max(v_scroll_bar_size[1]);

        min_size[0] += v_scroll_bar_size[0];
        min_size[1] += h_scroll_bar_size[1];

        min_size
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let this_rect = *ctx.get_rect(this);
        // let content_size = ctx.get_min_size(self.content);
        let this_width = this_rect[2] - this_rect[0];

        // assume that the vertical bar will be used
        let mut v_scroll_bar_size = ctx.get_min_size(self.v_scroll_bar)[0];

        // check if the horizontal bar is need
        let mut h_active;
        let mut h_scroll_bar_size;
        h_active = this_width - v_scroll_bar_size < self.content_width;
        h_scroll_bar_size = if h_active {
            ctx.get_min_size(self.h_scroll_bar)[1]
        } else {
            0.0
        };

        let mut view_rect = [
            this_rect[0],
            this_rect[1],
            this_rect[2] - v_scroll_bar_size,
            this_rect[3] - h_scroll_bar_size,
        ];

        // clamp delta_x
        let view_width = view_rect[2] - view_rect[0];
        if self.delta_x < 0.0 || view_width > self.content_width {
            self.delta_x = 0.0;
        } else if self.delta_x > self.content_width - view_width {
            self.delta_x = self.content_width - view_width;
        }

        // layout the items in the view
        self.create_items(view_rect, self.view, ctx);

        for (_, x) in self.last_created_items.iter() {
            if self.focused.as_ref().map_or(false, |f| x.id == f.id) {
                // hide the focused outside of the view
                println!("hide focused {}", x.id);
                ctx.set_designed_rect(
                    x.id,
                    [
                        view_rect[0] - self.delta_x,
                        view_rect[3] + 10.0,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        view_rect[3] + 110.0,
                    ],
                );
            } else {
                println!("remove {}", x.id);
                ctx.remove(x.id);
            }
        }
        self.last_created_items.clear();

        // if all the items are displayed in the view, there is no need for vertical bar
        let v_active =
            !(cmp_float(self.start_y, 0.0) && cmp_float(self.end_y, self.items.len() as f32));

        if !v_active {
            v_scroll_bar_size = 0.0;
            view_rect[2] = this_rect[2];
            // the first layout has assumed that the vertical bar exist. redo if it not exist.
            self.create_items_from_top(view_rect, self.view, ctx);

            // clamp delta_x
            let view_width = view_rect[2] - view_rect[0];
            if self.delta_x < 0.0 || view_width > self.content_width {
                self.delta_x = 0.0;
            } else if self.delta_x > self.content_width - view_width {
                self.delta_x = self.content_width - view_width;
            }

            // with the removal of the vertical bar, maybe the horizontal bar is not need anymore.
            h_active = this_width - v_scroll_bar_size < self.content_width;
            h_scroll_bar_size = if h_active {
                ctx.get_min_size(self.h_scroll_bar)[1]
            } else {
                0.0
            };
        }

        ctx.set_designed_rect(self.view, view_rect);

        // active and layout the horizontal and vertical bar as need
        if ctx.is_active(self.h_scroll_bar) {
            if !h_active {
                ctx.deactive(self.h_scroll_bar);
            }
        } else if h_active {
            ctx.active(self.h_scroll_bar);
        }

        if ctx.is_active(self.v_scroll_bar) {
            if !v_active {
                ctx.deactive(self.v_scroll_bar);
            }
        } else if v_active {
            ctx.active(self.v_scroll_bar);
        }

        if h_active {
            ctx.set_designed_rect(
                self.h_scroll_bar,
                [
                    this_rect[0],
                    this_rect[3] - h_scroll_bar_size,
                    this_rect[2] - v_scroll_bar_size,
                    this_rect[3],
                ],
            );
        }

        if v_active {
            ctx.set_designed_rect(
                self.v_scroll_bar,
                [
                    this_rect[2] - v_scroll_bar_size,
                    this_rect[1],
                    this_rect[2],
                    this_rect[3] - h_scroll_bar_size,
                ],
            );
        }

        // set the length of each bar handle
        if h_active {
            ctx.set_anchor_left(self.h_scroll_bar_handle, self.delta_x / self.content_width);
            ctx.set_anchor_right(
                self.h_scroll_bar_handle,
                ((self.delta_x + this_width) / self.content_width).min(1.0),
            );
        }

        if v_active {
            ctx.set_anchor_top(
                self.v_scroll_bar_handle,
                self.start_y / self.items.len() as f32,
            );
            ctx.set_anchor_bottom(
                self.v_scroll_bar_handle,
                (self.end_y / self.items.len() as f32).min(1.0),
            );
        }
    }
}