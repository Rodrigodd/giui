// TODO: call just once builder.item_count() per layout

use crate::BuilderContext;
use crate::{
    util::cmp_float, widgets::SetScrollPosition, Behaviour, Context, ControlBuilder, Id,
    InputFlags, KeyboardEvent, Layout, LayoutContext, MinSizeContext,
};

use std::{any::Any, collections::BTreeMap};
use winit::event::VirtualKeyCode;

use super::ScrollBar;

pub struct UpdateItems;

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
        let content = match ctx.get_active_children(this).get(0) {
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
    /// top position relative to the top of the view, when created
    y: f32,
    height: f32,
}
impl CreatedItem {
    fn new(id: Id, i: usize, y: f32, height: f32) -> Self {
        Self { id, i, y, height }
    }
}

#[allow(unused_variables)]
pub trait ListBuilder {
    /// This receive any event sent to the list control that was not handled.
    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {}

    /// The amount of items in the list. This can change dynamically.
    fn item_count(&mut self, ctx: &mut dyn BuilderContext) -> usize;

    /// Used to build the control of the item.
    ///
    /// The given ControlBuilder will have the list view set as parent. Any other created control
    /// should have the given ControlBuilder as its ancestor.
    fn create_item<'a>(
        &mut self,
        index: usize,
        list_id: Id,
        cb: ControlBuilder,
        ctx: &mut dyn BuilderContext,
    ) -> ControlBuilder;

    /// Used to update a previouly builded control of a item.
    ///
    /// The item_id is the Id of the control create in the last call of create_item for the given
    /// index. If this function returns true, the control is said to be updated, otherwise, if
    /// false, the control is removed and a new on is created, by calling create_item immediately
    /// afterwards.
    #[must_use]
    fn update_item(&mut self, index: usize, item_id: Id, ctx: &mut dyn BuilderContext) -> bool {
        true
    }

    /// Called after all items has been updated.
    ///
    /// In the case where the items need to be updated sometimes, this can be used to mark all
    /// items as updated at once, intead of keeping a update flag for each item.
    fn finished_layout(&mut self) {}
}

pub struct List<C: ListBuilder> {
    space: f32,
    margins: [f32; 4],
    content_width: f32,
    /// The amount of horizontal scroll in pixels.
    delta_x: f32,
    /// The amount of horizontal scroll in the last layout.
    last_delta_x: f32,
    /// The amount of vertical scroll, between 0 and 1, for the next layout
    set_y: Option<f32>,
    /// The variation of vertical scroll, in items, for the next layout
    delta_y: f32,
    /// The position of the top of the view, in items
    start_y: f32,
    /// The position of the bottom of the view, in items
    end_y: f32,
    /// The rect for the view, in the last layout
    last_rect: [f32; 4],
    view: Id,
    v_scroll_bar: Id,
    v_scroll_bar_handle: Id,
    h_scroll_bar: Id,
    h_scroll_bar_handle: Id,
    last_created_items: BTreeMap<usize, CreatedItem>,
    created_items: BTreeMap<usize, CreatedItem>,
    // TODO: the focused really need to be a CreatedItem, or can it be a usize for which the
    // CreatedItem is in last_created_items?
    focused: Option<CreatedItem>,
    builder: C,
}
impl<C: ListBuilder> List<C> {
    /// v_scroll must be a descendant of this
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        content_width: f32,
        spacing: f32,
        margins: [f32; 4],
        view: Id,
        v_scroll_bar: Id,
        v_scroll_bar_handle: Id,
        h_scroll_bar: Id,
        h_scroll_bar_handle: Id,
        builder: C,
    ) -> Self {
        Self {
            // TODO: spacing and margins must be paramenters
            space: spacing,
            margins,
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
            focused: None,
            last_created_items: BTreeMap::new(),
            created_items: BTreeMap::new(),
            builder,
        }
    }

    fn create_item_generic(
        &mut self,
        i: usize,
        list_id: Id,
        y: impl FnOnce(f32) -> f32,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
        from_bottom: bool,
    ) -> f32 {
        let focused;
        let mut x = if self.focused.as_ref().map_or(false, |x| x.i == i) {
            focused = true;
            let x = self.focused.take().unwrap();
            self.last_created_items.remove(&i);
            if self.builder.update_item(i, x.id, ctx) {
                ctx.recompute_min_size(x.id);
                if !from_bottom {
                    ctx.move_to_front(x.id);
                }
                log::trace!("move focused {}", x.id);
                x
            } else {
                ctx.remove(x.id);
                let id = self
                    .builder
                    .create_item(i, list_id, ctx.create_control(), ctx)
                    .parent(self.view)
                    .build(ctx);
                log::trace!("recreate focused {} as {}", x.id, id);
                CreatedItem::new(id, i, 0.0, 0.0)
            }
        } else {
            focused = false;
            match self.last_created_items.remove(&i) {
                Some(x) => {
                    if self.builder.update_item(i, x.id, ctx) {
                        ctx.recompute_min_size(x.id);
                        if !from_bottom {
                            ctx.move_to_front(x.id);
                        }
                        log::trace!("move {}", x.id);
                        x
                    } else {
                        ctx.remove(x.id);
                        let id = self
                            .builder
                            .create_item(i, list_id, ctx.create_control(), ctx)
                            .parent(self.view)
                            .build(ctx);
                        log::trace!("recreate {} as {}", x.id, id);
                        CreatedItem::new(id, i, 0.0, 0.0)
                    }
                }
                None => {
                    let id = self
                        .builder
                        .create_item(i, list_id, ctx.create_control(), ctx)
                        .parent(self.view)
                        .build(ctx);
                    log::trace!("create {}", id);
                    CreatedItem::new(id, i, 0.0, 0.0)
                }
            }
        };
        if from_bottom {
            ctx.move_to_back(x.id);
        }

        let id = x.id;

        let top_margin = if i == 0 { self.margins[1] } else { 0.0 };
        let bottom_margin = if i + 1 == self.builder.item_count(ctx) {
            self.margins[3]
        } else {
            self.space
        };
        assert!(ctx.get_min_size(id)[1] != 0.0);
        let height = ctx.get_min_size(id)[1] + top_margin + bottom_margin;
        let y = y(height);
        ctx.set_designed_rect(
            id,
            [
                view_rect[0] + self.margins[0] - self.delta_x,
                y + top_margin,
                (view_rect[2]).max(view_rect[0] + self.content_width)
                    - self.margins[2]
                    - self.delta_x,
                y + height - bottom_margin,
            ],
        );
        x.y = y - view_rect[1];
        x.height = height;
        if focused {
            self.focused = Some(x.clone());
        }
        self.created_items.insert(i, x);
        height
    }

    fn create_item(
        &mut self,
        i: usize,
        list_id: Id,
        y: f32,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        self.create_item_generic(i, list_id, |_| y, ctx, view_rect, false)
    }

    fn create_item_at(
        &mut self,
        start_y: f32,
        list_id: Id,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        let i = start_y as usize;
        let mut y = 0.0;
        let height = self.create_item_generic(
            i,
            list_id,
            |height| {
                y = view_rect[1] - start_y.fract() * height;
                y
            },
            ctx,
            view_rect,
            false,
        );
        y += height;
        y
    }

    fn create_item_from_bottom(
        &mut self,
        i: usize,
        list_id: Id,
        y: f32,
        ctx: &mut LayoutContext,
        view_rect: [f32; 4],
    ) -> f32 {
        self.create_item_generic(i, list_id, |heigth| y - heigth, ctx, view_rect, true)
    }

    fn create_items_from_top(&mut self, view_rect: [f32; 4], list_id: Id, ctx: &mut LayoutContext) {
        // log::trace!("create from top!");

        self.last_created_items.append(&mut self.created_items);

        let mut i = 0;
        self.start_y = 0.0;
        self.delta_y = 0.0;

        let mut height;
        let mut y = view_rect[1];

        let item_count = self.builder.item_count(ctx);

        // create items below, if necessary
        while y <= view_rect[3] {
            // there is not enough items to fill the view
            if i >= item_count {
                self.end_y = item_count as f32;
                // log::trace!("end at {}", self.end_y);
                return;
            }
            height = self.create_item(i, list_id, y, ctx, view_rect);
            y += height;
            i += 1;
        }

        {
            let last = self.created_items.iter().rev().next().unwrap().1;

            let y = view_rect[1] + last.y;
            let gap = (view_rect[3] - y) / last.height;
            if !(0.0..1.0).contains(&gap) {
                log::error!("gap: {}, height: {}, y: {}", gap, last.height, last.y);
            }
            self.end_y = last.i as f32 + gap;
            // log::trace!("end at {}", self.end_y);
        }
    }

    fn create_items_from_bottom(
        &mut self,
        view_rect: [f32; 4],
        list_id: Id,
        ctx: &mut LayoutContext,
    ) {
        self.last_rect = view_rect;
        self.end_y = self.builder.item_count(ctx) as f32;

        // log::trace!("create items from_bottom");

        self.last_created_items.append(&mut self.created_items);

        let mut i = self.builder.item_count(ctx) - 1;
        let mut y = view_rect[3];

        while y > view_rect[1] {
            let height = self.create_item_from_bottom(i, list_id, y, ctx, view_rect);
            y -= height;
            // if the top item don't fill the view yet, create the items from top
            if i == 0 {
                if y > view_rect[1] {
                    return self.create_items_from_top(view_rect, list_id, ctx);
                }
                break;
            }
            i -= 1;
        }

        {
            let first = self.created_items.iter().next().unwrap().1;

            let gap = -first.y / first.height;
            if !(0.0..1.0).contains(&gap) {
                log::error!("gap: {}, height: {}, y: {}", gap, first.height, first.y);
            }
            self.start_y = first.i as f32 + gap;
        }
    }

    // TODO: make this reuse already created items
    fn create_items_from_a_start_y(
        &mut self,
        start_y: f32,
        view_rect: [f32; 4],
        list_id: Id,
        ctx: &mut LayoutContext,
    ) {
        if cmp_float(start_y, 0.0) {
            return self.create_items_from_top(view_rect, list_id, ctx);
        }
        // log::trace!("create from zero!");

        self.start_y = start_y;
        let mut i = self.start_y as usize;

        if i >= self.builder.item_count(ctx) {
            return self.create_items_from_bottom(view_rect, list_id, ctx);
        }

        let mut y = self.create_item_at(start_y, list_id, ctx, view_rect);
        i += 1;

        if i >= self.builder.item_count(ctx) && y < view_rect[3] {
            return self.create_items_from_bottom(view_rect, list_id, ctx);
        }

        while y <= view_rect[3] {
            let height = self.create_item(i, list_id, y, ctx, view_rect);
            y += height;
            i += 1;
            if i >= self.builder.item_count(ctx) {
                if y < view_rect[3] {
                    return self.create_items_from_bottom(view_rect, list_id, ctx);
                }
                break;
            }
        }

        {
            let last = self.created_items.iter().rev().next().unwrap().1;

            let y = view_rect[1] + last.y;
            let gap = (view_rect[3] - y) / last.height;
            if !(0.0..1.0).contains(&gap) {
                log::error!("gap: {}, height: {}, y: {}", gap, last.height, last.y);
            }
            self.end_y = last.i as f32 + gap;
            log::trace!("end at {}", self.end_y);
        }
    }

    fn create_items(&mut self, view_rect: [f32; 4], list_id: Id, ctx: &mut LayoutContext) {
        let same_rect = cmp_float(view_rect[0], self.last_rect[0])
            && cmp_float(view_rect[1], self.last_rect[1])
            && cmp_float(view_rect[2], self.last_rect[2])
            && cmp_float(view_rect[3], self.last_rect[3]);
        // log::trace!("delta_y: {}", self.delta_y);
        if same_rect
            && cmp_float(0.0, self.delta_y)
            && self.set_y.is_none()
            && cmp_float(self.last_delta_x, self.delta_x)
            && self.builder.item_count(ctx) > 0
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
            self.create_items_from_top(view_rect, list_id, ctx);
        } else if let Some(y) = self.set_y.take() {
            self.create_items_from_a_start_y(y, view_rect, list_id, ctx);
        } else {
            if delta_y < 0.0 {
                // create items above
                let mut i = self.start_y as usize;
                let start_control = self.last_created_items.get(&i).unwrap();
                let mut y = start_control.y + view_rect[1] - delta_y;
                while y > view_rect[1] {
                    if i == 0 {
                        return self.create_items_from_top(view_rect, list_id, ctx);
                    }
                    i -= 1;
                    let height = self.create_item_from_bottom(i, list_id, y, ctx, view_rect);
                    y -= height;
                }
            }

            let mut i = self.start_y as usize;
            let start_control = self.last_created_items.get(&i).unwrap();
            let mut y = start_control.y + view_rect[1] - delta_y;

            if i >= self.builder.item_count(ctx) && y < view_rect[3] {
                return self.create_items_from_bottom(view_rect, list_id, ctx);
            }

            // create items below, if necessary
            while y <= view_rect[3] {
                let height = self.create_item(i, list_id, y, ctx, view_rect);
                y += height;
                i += 1;
                if i >= self.builder.item_count(ctx) {
                    if y < view_rect[3] {
                        return self.create_items_from_bottom(view_rect, list_id, ctx);
                    }
                    break;
                }
            }

            // destroy items above, if any
            loop {
                let (&i, item) = self.created_items.iter().next().unwrap();
                if item.y + item.height > 0.0 {
                    break;
                }
                // give item back to last_created_items
                let item = self.created_items.remove(&i).unwrap();
                let removed = self.last_created_items.insert(i, item);
                debug_assert!(removed.is_none());
            }

            // destroy items below, if any
            loop {
                let (&i, item) = self.created_items.iter().next_back().unwrap();
                if item.y <= view_rect[3] - view_rect[1] {
                    break;
                }
                // give item back to last_created_items
                let item = self.created_items.remove(&i).unwrap();
                let removed = self.last_created_items.insert(i, item);
                debug_assert!(removed.is_none());
            }

            {
                let first = self.created_items.iter().next().unwrap().1;

                let gap = -first.y / first.height;
                if !(0.0..1.0).contains(&gap) {
                    log::error!("gap: {}, height: {}, y: {}", gap, first.height, first.y);
                }
                self.start_y = first.i as f32 + gap;
            }
            {
                let last = self.created_items.iter().rev().next().unwrap().1;

                let mut gap = (view_rect[3] - view_rect[1] - last.y) / last.height;
                if !(0.0..1.0).contains(&gap) {
                    log::error!("gap: {}, height: {}, y: {}", gap, last.height, last.y);
                }
                gap = gap.clamp(0.0, 1.0 - f32::EPSILON);
                self.end_y = last.i as f32 + gap;
            }
            // log::trace!("start at {}", self.start_y);
            // log::trace!("end at {}", self.end_y);
        }
    }
}
impl<C: ListBuilder> Behaviour for List<C> {
    fn on_start(&mut self, _this: Id, ctx: &mut Context) {
        ctx.move_to_front(self.h_scroll_bar);
        ctx.move_to_front(self.v_scroll_bar);
    }

    fn on_focus_change(&mut self, focus: bool, this: Id, ctx: &mut Context) {
        if focus {
            self.focused = self
                .created_items
                .iter()
                .find(|(_, x)| ctx.is_focus(x.id))
                .map(|(_, x)| x)
                .cloned();
            if let Some(focused) = &self.focused {
                let view_rect = ctx.get_rect(self.view);
                let view_height = view_rect[3] - view_rect[1];
                if focused.y + focused.height >= view_height {
                    self.delta_y += focused.y + focused.height - view_height + 10.0;
                } else if focused.y <= 0.0 {
                    self.delta_y -= -focused.y + 10.0;
                }
                ctx.dirty_layout(this);
            }
        } else {
            self.focused = None;
        }
    }

    fn on_active(&mut self, _this: Id, _ctx: &mut Context) {
        // let view_rect = ctx.get_rect(self.view);

        // let view_width = view_rect[2] - view_rect[0];
        // let view_height = view_rect[3] - view_rect[1];

        // let handle_min_width = ctx.get_min_size(self.h_scroll_bar_handle)[0];
        // let handle_min_height = ctx.get_min_size(self.v_scroll_bar_handle)[1];

        // let mut start = self.delta_x / self.content_width;
        // let mut end = ((self.delta_x + view_width) / self.content_width).min(1.0);
        // let gap = handle_min_width - (end - start) * view_width;

        // if gap > 0.0 {
        //     start *= 1.0 - gap / view_width;
        //     end *= 1.0 - gap / view_width;
        // }

        // ctx.set_anchor_left(self.h_scroll_bar_handle, start);
        // ctx.set_anchor_right(self.h_scroll_bar_handle, end);

        // let mut start = self.start_y / self.create_item.item_count() as f32;
        // let mut end = (self.end_y / self.create_item.item_count() as f32).min(1.0);
        // let gap = handle_min_height - (end - start) * view_height;

        // if gap > 0.0 {
        //     start *= 1.0 - gap / view_height;
        //     end *= 1.0 - gap / view_height;
        // }

        // ctx.set_anchor_top(self.v_scroll_bar_handle, start);
        // ctx.set_anchor_bottom(self.v_scroll_bar_handle, end);
    }

    fn on_event(&mut self, event: Box<dyn Any>, this: Id, ctx: &mut Context) {
        if let Some(event) = event.downcast_ref::<SetScrollPosition>() {
            if !event.vertical {
                let total_size = self.content_width - ctx.get_size(self.view)[0];
                self.delta_x = event.value.max(0.0) * total_size;
            } else {
                let total_size = self.builder.item_count(ctx) as f32 - (self.end_y - self.start_y);
                self.set_y = Some(event.value.max(0.0) * total_size);
                // log::trace!("set y to {:?}", self.set_y);
            }
            ctx.dirty_layout(self.view);
            ctx.dirty_layout(this);
        } else if event.is::<UpdateItems>() {
            // TODO: I add this set_y here, to force a update, but i don't know if this will go
            // wrong!!
            log::trace!("update list items");
            self.set_y = Some(self.start_y);
            ctx.dirty_layout(this);
        } else {
            self.builder.on_event(event, this, ctx)
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
        if cmp_float(self.start_y, 0.0)
            && cmp_float(self.end_y, self.builder.item_count(ctx) as f32)
        {
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
impl<C: ListBuilder> Layout for List<C> {
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
        let this_rect = ctx.get_rect(this);
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
        self.create_items(view_rect, this, ctx);
        self.builder.finished_layout();

        for (_, x) in self.last_created_items.iter() {
            if self.focused.as_ref().map_or(false, |f| x.id == f.id) {
                // hide the focused outside of the view
                log::trace!("hide focused {}", x.id);
                ctx.set_designed_rect(
                    x.id,
                    [
                        view_rect[0] + self.margins[0] - self.delta_x,
                        view_rect[3] + 1010.0,
                        (view_rect[2]).max(view_rect[0] + self.content_width)
                            - self.margins[2]
                            - self.delta_x,
                        view_rect[3] + 1110.0,
                    ],
                );
            } else {
                log::trace!("remove {}", x.id);
                ctx.remove(x.id);
            }
        }
        self.last_created_items.clear();

        // if all the items are displayed in the view, there is no need for vertical bar
        let v_active = !(cmp_float(self.start_y, 0.0)
            && cmp_float(self.end_y, self.builder.item_count(ctx) as f32));

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
            let view_width = view_rect[2] - view_rect[0];

            let start = self.delta_x / self.content_width;
            let end = ((self.delta_x + view_width) / self.content_width).min(1.0);

            ScrollBar::set_anchors(ctx, self.h_scroll_bar_handle, false, start, end, view_width);
        }

        if v_active {
            let view_height = view_rect[3] - view_rect[1];

            let start = self.start_y / self.builder.item_count(ctx) as f32;
            let end = (self.end_y / self.builder.item_count(ctx) as f32).min(1.0);

            ScrollBar::set_anchors(ctx, self.v_scroll_bar_handle, true, start, end, view_height);
        }
    }
}
