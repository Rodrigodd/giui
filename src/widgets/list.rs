use crate::{
    util::cmp_float, widgets::SetScrollPosition, Behaviour, Context, ControlBuilder, Id,
    InputFlags, KeyboardEvent, Layout, LayoutContext, MinSizeContext,
};

use std::any::Any;
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

#[derive(Debug)]
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
    created_items: Vec<CreatedItem>,
    focused: Option<Id>,
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
            created_items: Vec::new(),
            create_item,
        }
    }

    fn create_items_from_top(&mut self, view_rect: [f32; 4], this: Id, ctx: &mut LayoutContext) {
        println!("create from top!");

        let mut i = 0;
        self.start_y = 0.0;
        self.delta_y = 0.0;

        let mut stop = usize::max_value();
        let mut height = 0.0;
        let mut y = view_rect[1];
        let mut created_items = Vec::new();
        if !self.created_items.is_empty() {
            // create items above, if necessary
            while y < view_rect[3] && i != self.created_items[0].i {
                // there is not enough items to fill the view
                if i >= self.items.len() {
                    self.end_y = self.items.len() as f32;
                    println!("end at {}", self.end_y);
                    return;
                }
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                created_items.push(CreatedItem::new(id, i, y - view_rect[1], height));
                y += self.spacing + height;
                i += 1;
            }

            debug_assert_eq!(
                self.created_items[0].i,
                i,
                "y < view_rect[3]: {}",
                y < view_rect[3]
            );

            if self.created_items[0].i == i {
                for (index, created_item) in self.created_items.iter_mut().enumerate() {
                    if y >= view_rect[3] {
                        stop = index;
                        break;
                    }
                    let id = created_item.id;
                    println!("move {}", id);
                    height = ctx.get_min_size(id)[1];
                    ctx.set_designed_rect(
                        id,
                        [
                            view_rect[0] - self.delta_x,
                            y,
                            view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                            y + height,
                        ],
                    );
                    created_item.y = y - view_rect[1];
                    y += self.spacing + height;
                    i += 1;
                }
            } else {
                // created items is out of visible range
                for x in self.created_items.drain(..) {
                    println!("remove {}", x.id);
                    ctx.remove(x.id);
                }
            }
        }

        // remove created items below, if any
        if stop != usize::max_value() {
            for to_remove in self.created_items.drain(stop..) {
                println!("remove {}", to_remove.id);
                ctx.remove(to_remove.id);
            }
        } else {
            // create items below, if necessary
            while y < view_rect[3] {
                // there is not enough items to fill the view
                if i >= self.items.len() {
                    self.created_items.splice(0..0, created_items);
                    self.end_y = self.items.len() as f32;
                    println!("end at {}", self.end_y);
                    return;
                }
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                height = ctx.get_min_size(id)[1];
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
                    .push(CreatedItem::new(id, i, y - view_rect[1], height));
                y += self.spacing + height;
                i += 1;
            }
        }

        self.created_items.splice(0..0, created_items);

        self.end_y =
            (i - 1) as f32 + (view_rect[3] - (y - height - self.spacing)) / (height + self.spacing);
        println!("end at {}", self.end_y);
    }

    fn create_items_from_bottom(&mut self, view_rect: [f32; 4], this: Id, ctx: &mut LayoutContext) {
        self.last_rect = view_rect;
        self.end_y = self.items.len() as f32;

        println!("create items from_bottom");

        let mut i = self.items.len() - 1;
        let mut y = view_rect[3];
        let mut height;

        if let Some(last_created_item) = self.created_items.last() {
            let mut created_items = Vec::new();
            // create items at bottom
            while y > view_rect[1] && i != last_created_item.i {
                let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                println!("create {}", id);
                ctx.move_to_front(id);
                height = ctx.get_min_size(id)[1];
                y -= height;
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                created_items.push(CreatedItem::new(id, i, y - view_rect[1], height));
                y -= self.spacing;
                if i == 0 {
                    return self.create_items_from_top(view_rect, this, ctx);
                }
                i -= 1;
            }
            if i == last_created_item.i {
                // move already created items
                let mut stop = usize::max_value();
                for (index, created_item) in self.created_items.iter_mut().enumerate().rev() {
                    if y + self.spacing < view_rect[1] {
                        stop = index;
                        break;
                    }
                    let id = created_item.id;
                    println!("move {}", id);
                    height = ctx.get_min_size(id)[1];
                    y -= height;
                    ctx.set_designed_rect(
                        id,
                        [
                            view_rect[0] - self.delta_x,
                            y,
                            view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                            y + height,
                        ],
                    );
                    created_item.y = y - view_rect[1];
                    y -= self.spacing;
                    // if the top item don't fill the view yet, create the items from top
                    if i == 0 {
                        if y + self.spacing >= view_rect[1] {
                            return self.create_items_from_top(view_rect, this, ctx);
                        }
                        break;
                    }
                    i -= 1;
                }
                println!("stop: {}", stop);
                // remove items above, if any
                if stop != usize::max_value() {
                    for x in self.created_items.drain(0..=stop) {
                        println!("remove {}", x.id);
                        ctx.remove(x.id);
                    }
                }
            } else {
                // created items is out of visible range
                for x in self.created_items.drain(..) {
                    println!("remove {}", x.id);
                    ctx.remove(x.id);
                }
            }
            self.created_items.extend(created_items.into_iter().rev());
        }

        let mut created_items = Vec::new();
        while y > view_rect[1] {
            let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
            println!("create {}", id);
            ctx.move_to_front(id);
            height = ctx.get_min_size(id)[1];
            y -= height;
            ctx.set_designed_rect(
                id,
                [
                    view_rect[0] - self.delta_x,
                    y,
                    view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                    y + height,
                ],
            );
            created_items.push(CreatedItem::new(id, i, y - view_rect[1], height));
            y -= self.spacing;
            // if the top item don't fill the view yet, create the items from top
            if i == 0 {
                if y > view_rect[1] {
                    return self.create_items_from_top(view_rect, this, ctx);
                }
                break;
            }
            i -= 1;
        }

        self.created_items
            .splice(0..0, created_items.into_iter().rev());

        {
            println!("created_items: {:?}", self.created_items);
            let id = self.created_items[0].id;
            let height = ctx.get_min_size(id)[1];
            let i = self.created_items[0].i;
            let y = self.created_items[0].y + view_rect[1];

            self.start_y = (i as f32) + (view_rect[1] - y) / (height + self.spacing);
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

        for x in self.created_items.drain(..) {
            println!("remove {}", x.id);
            ctx.remove(x.id);
        }

        self.start_y = start_y;
        let mut i = self.start_y as usize;

        if i >= self.items.len() {
            return self.create_items_from_bottom(view_rect, this, ctx);
        }

        let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
        println!("create {}", id);
        let mut height = ctx.get_min_size(id)[1];
        let mut y = view_rect[1] - self.start_y.fract() * (height + self.spacing);
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
            .push(CreatedItem::new(id, i, y - view_rect[1], height));
        y += self.spacing + height;
        i += 1;

        if i >= self.items.len() && y - self.spacing < view_rect[3] {
            return self.create_items_from_bottom(view_rect, this, ctx);
        }

        while y < view_rect[3] {
            let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
            println!("create {}", id);
            height = ctx.get_min_size(id)[1];
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
                .push(CreatedItem::new(id, i, y - view_rect[1], height));
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
            let last = self.created_items.last().unwrap();
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
        self.last_rect = view_rect;
        self.last_delta_x = self.delta_x;
        let delta_y = self.delta_y;
        self.delta_y = 0.0;

        if self.created_items.is_empty() {
            self.create_items_from_a_start_y(0.0, view_rect, this, ctx);
        } else if let Some(y) = self.set_y.take() {
            self.create_items_from_a_start_y(y, view_rect, this, ctx);
        } else {
            let mut created_items = Vec::new();
            if delta_y < 0.0 {
                // create items above
                let mut i = self.created_items[0].i;
                let mut y = self.created_items[0].y + view_rect[1] - delta_y - self.spacing;
                if i == 0 {
                    y += self.spacing;
                }
                let mut height;
                while y > view_rect[1] {
                    if i == 0 {
                        self.created_items
                            .splice(0..0, created_items.into_iter().rev());
                        return self.create_items_from_top(view_rect, this, ctx);
                    }
                    i -= 1;
                    let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                    println!("create {}", id);
                    ctx.move_to_front(id);
                    height = ctx.get_min_size(id)[1];
                    y -= height;
                    ctx.set_designed_rect(
                        id,
                        [
                            view_rect[0] - self.delta_x,
                            y,
                            view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                            y + height,
                        ],
                    );
                    created_items.push(CreatedItem::new(id, i, y - view_rect[1], height));
                    if i != 0 {
                        y -= self.spacing;
                    }
                }
                created_items.reverse();
            }

            let mut y = self.created_items[0].y - delta_y + view_rect[1];
            let mut i = self.created_items[0].i;
            let mut height;

            // move created items
            let mut stop = usize::max_value();
            for (index, created_item) in self.created_items.iter_mut().enumerate() {
                if y >= view_rect[3] {
                    stop = index;
                    break;
                }
                let id = created_item.id;
                println!("move {}", id);
                height = ctx.get_min_size(id)[1];
                ctx.set_designed_rect(
                    id,
                    [
                        view_rect[0] - self.delta_x,
                        y,
                        view_rect[2].max(view_rect[0] + self.content_width) - self.delta_x,
                        y + height,
                    ],
                );
                created_item.y = y - view_rect[1];
                y += self.spacing + height;
                i += 1;
            }

            // remove created items below, if any
            if stop != usize::max_value() {
                for to_remove in self.created_items.drain(stop..) {
                    println!("remove {}", to_remove.id);
                    ctx.remove(to_remove.id);
                }
            } else {
                if i >= self.items.len() && y - self.spacing < view_rect[3] {
                    return self.create_items_from_bottom(view_rect, this, ctx);
                }

                // create items below, if necessary
                while y < view_rect[3] {
                    let id = (self.create_item)(&self.items[i], this, ctx.create_control()).build();
                    println!("create {}", id);
                    height = ctx.get_min_size(id)[1];
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
                        .push(CreatedItem::new(id, i, y - view_rect[1], height));
                    y += self.spacing + height;
                    i += 1;
                    if i >= self.items.len() {
                        if y - self.spacing < view_rect[3] {
                            return self.create_items_from_bottom(view_rect, this, ctx);
                        }
                        break;
                    }
                }
            }

            self.created_items.splice(0..0, created_items);

            debug_assert!(self.created_items.windows(2).all(|x| x[0].i <= x[1].i));
            debug_assert!(self.created_items.windows(2).all(|x| x[0].y <= x[1].y));

            // remove created items above, if any
            let x = self
                .created_items
                .iter()
                .position(|x| x.y + x.height + self.spacing > 0.0)
                .unwrap_or(0);
            for to_remove in self.created_items.drain(0..x) {
                println!("remove {}", to_remove.id);
                ctx.remove(to_remove.id);
            }

            println!("items: {:?}", self.created_items);

            {
                let id = self.created_items[0].id;
                let height = ctx.get_min_size(id)[1];
                let i = self.created_items[0].i;
                let y = self.created_items[0].y + view_rect[1];

                self.start_y = i as f32 + (view_rect[1] - y) / (height + self.spacing);
            }
            {
                let last = self.created_items.last().unwrap();
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
                .find(|x| ctx.is_focus(x.id))
                .map(|x| x.id);
        } else {
            self.focused = None;
        }
    }

    fn on_active(&mut self, _this: Id, ctx: &mut Context) {
        // let content_size = ctx.get_min_size(self.content);

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
            for x in self.created_items.drain(..) {
                ctx.remove(x.id);
            }
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
