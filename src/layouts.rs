use crate::{
    context::{LayoutContext, MinSizeContext},
    Id, Layout,
};

pub struct FitText;
impl Layout for FitText {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let fonts = ctx.get_fonts();
        let min_size = ctx
            .get_graphic(this)
            .compute_min_size(fonts)
            .unwrap_or([0.0, 0.0]);
        ctx.set_this_min_size(min_size);
    }
    fn update_layouts(&mut self, _: Id, _: &mut LayoutContext) {}
}

pub struct MarginLayout {
    margins: [f32; 4],
}
impl MarginLayout {
    pub fn new(margins: [f32; 4]) -> Self {
        Self { margins }
    }
}
impl Layout for MarginLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let mut min_size = [0.0f32, 0.0];
        for child in ctx.get_children(this) {
            let c_min_size = ctx.get_layouting(child).get_min_size();
            min_size[0] = min_size[0].max(c_min_size[0]);
            min_size[1] = min_size[1].max(c_min_size[1]);
        }
        ctx.set_this_min_size([
            self.margins[0] + self.margins[2] + min_size[0],
            self.margins[1] + self.margins[3] + min_size[1],
        ]);
    }
    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let rect = ctx.get_layouting(this).get_rect();
        let des_rect = [
            rect[0] + self.margins[0],
            rect[1] + self.margins[1],
            rect[2] - self.margins[2],
            rect[3] - self.margins[3],
        ];
        for child in ctx.get_children(this) {
            ctx.set_designed_rect(child, des_rect);
        }
    }
}

pub struct RatioLayout {
    ratio: f32,
    align: (i8, i8),
}
impl RatioLayout {
    pub fn new(ratio: f32, align: (i8, i8)) -> Self {
        Self { ratio, align }
    }
}
impl Layout for RatioLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let mut min_size = [0.0f32, 0.0];
        for child in ctx.get_children(this) {
            let c_min_size = ctx.get_layouting(child).get_min_size();
            min_size[0] = min_size[0].max(c_min_size[0]);
            min_size[1] = min_size[1].max(c_min_size[1]);
        }
        if min_size[0] > min_size[1] * self.ratio {
            ctx.set_this_min_size([min_size[0], min_size[0] / self.ratio]);
        } else {
            ctx.set_this_min_size([min_size[1] * self.ratio, min_size[1]]);
        }
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let rect = ctx.get_layouting(this);
        let mut x = rect.get_rect()[0];
        let mut y = rect.get_rect()[1];
        let des_rect = if rect.get_width() < rect.get_height() * self.ratio {
            match self.align.1 {
                -1 => {}
                0 => y += (rect.get_height() - rect.get_width() / self.ratio) / 2.0,
                1 => y += rect.get_height() - rect.get_width() / self.ratio,
                _ => {}
            }
            [
                x,
                y,
                x + rect.get_width(),
                y + rect.get_width() / self.ratio,
            ]
        } else {
            match self.align.0 {
                -1 => {}
                0 => x += (rect.get_width() - rect.get_height() * self.ratio) / 2.0,
                1 => x += rect.get_width() - rect.get_height() * self.ratio,
                _ => {}
            }
            [
                x,
                y,
                x + rect.get_height() * self.ratio,
                y + rect.get_height(),
            ]
        };
        for child in ctx.get_children(this) {
            ctx.set_designed_rect(child, des_rect);
        }
    }
}

pub struct HBoxLayout {
    spacing: f32,
    margins: [f32; 4],
    align: i8,
}
impl HBoxLayout {
    pub fn new(spacing: f32, margins: [f32; 4], align: i8) -> Self {
        Self {
            spacing,
            margins,
            align,
        }
    }
}
impl Layout for HBoxLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            ctx.set_this_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let mut min_width: f32 =
                self.margins[0] + self.margins[2] + (children.len() - 1) as f32 * self.spacing;
            let mut min_height: f32 = 0.0;
            for child in children {
                let [width, height] = ctx.get_layouting(child).get_min_size();
                min_width += width;
                min_height = min_height.max(height);
            }
            ctx.set_this_min_size([min_width, min_height + self.margins[1] + self.margins[3]]);
        }
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            return;
        }
        let mut reserved_width = self.spacing * (children.len() - 1) as f32;
        let mut max_weight = 0.0;
        for child in children {
            let rect = ctx.get_layouting(child);
            reserved_width += rect.get_min_size()[0];
            if rect.is_expand_x() {
                max_weight += rect.ratio_x;
            }
        }
        let rect = ctx.get_layouting(this);
        let width = rect.get_width() - self.margins[0] - self.margins[2];
        let rect = *rect.get_rect();
        let top = rect[1] + self.margins[1];
        let bottom = rect[3] - self.margins[3];
        let mut x = rect[0] + self.margins[0];
        let free_width = width - reserved_width;
        if free_width <= 0.0 || max_weight == 0.0 {
            match self.align {
                0 => x += free_width / 2.0,
                1 => x += free_width,
                _ => {}
            }
            for child in ctx.get_children(this) {
                let min_width = ctx.get_min_size(child)[0];
                ctx.set_designed_rect(child, [x, top, x + min_width, bottom]);
                x += self.spacing + min_width;
            }
        } else {
            for child in ctx.get_children(this) {
                let rect = ctx.get_layouting(child);
                if rect.is_expand_x() {
                    // FIXME: this implementation imply that rect with same ratio,
                    // may not have the same size when expanded
                    let width = rect.get_min_size()[0] + free_width * rect.ratio_x / max_weight;
                    ctx.set_designed_rect(child, [x, top, x + width, bottom]);
                    x += self.spacing + width
                } else {
                    let width = rect.get_min_size()[0];
                    ctx.set_designed_rect(child, [x, top, x + width, bottom]);
                    x += self.spacing + width;
                }
            }
        }
    }
}

pub struct VBoxLayout {
    spacing: f32,
    margins: [f32; 4],
    align: i8,
}
impl VBoxLayout {
    pub fn new(spacing: f32, margins: [f32; 4], align: i8) -> Self {
        Self {
            spacing,
            margins,
            align,
        }
    }
}
impl Layout for VBoxLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            ctx.set_this_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let mut min_width: f32 = 0.0;
            let mut min_height: f32 =
                self.margins[1] + self.margins[3] + (children.len() - 1) as f32 * self.spacing;
            for child in children {
                let [width, height] = ctx.get_layouting(child).get_min_size();
                min_width = min_width.max(width);
                min_height += height;
            }
            ctx.set_this_min_size([min_width + self.margins[0] + self.margins[2], min_height]);
        }
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            return;
        }
        let mut reserved_height = self.spacing * (children.len() - 1) as f32;
        let mut max_weight = 0.0;
        for child in children {
            let rect = ctx.get_layouting(child);
            reserved_height += rect.get_min_size()[1];
            if rect.is_expand_y() {
                max_weight += rect.ratio_y;
            }
        }
        let rect = ctx.get_layouting(this);
        let height = rect.get_height() - self.margins[1] - self.margins[3];
        let rect = *rect.get_rect();
        let left = rect[0] + self.margins[0];
        let right = rect[2] - self.margins[2];
        let mut y = rect[1] + self.margins[1];
        let free_height = height - reserved_height;
        if free_height <= 0.0 || max_weight == 0.0 {
            match self.align {
                0 => y += free_height / 2.0,
                1 => y += free_height,
                _ => {}
            }
            for child in ctx.get_children(this) {
                let height = ctx.get_min_size(child)[1];
                ctx.set_designed_rect(child, [left, y, right, y + height]);
                y += self.spacing + height;
            }
        } else {
            for child in ctx.get_children(this) {
                let rect = ctx.get_layouting(child);
                if rect.is_expand_y() {
                    // FIXME: this implementation imply that rect with same ratio,
                    // may not have the same size when expanded
                    let height = rect.get_min_size()[1] + free_height * rect.ratio_y / max_weight;
                    ctx.set_designed_rect(child, [left, y, right, y + height]);
                    y += self.spacing + height;
                } else {
                    let height = rect.get_min_size()[1];
                    ctx.set_designed_rect(child, [left, y, right, y + height]);
                    y += self.spacing + height;
                }
            }
        }
    }
}

pub struct GridLayout {
    spacing: [f32; 2],
    margins: [f32; 4],
    columns: u32,
    rows: u32,
    min_sizes: Vec<f32>,
    expand: Vec<bool>,
    weights: Vec<f32>,
}
impl GridLayout {
    pub fn new(spacing: [f32; 2], margins: [f32; 4], columns: u32) -> Self {
        Self {
            spacing,
            margins,
            columns,
            rows: 0,
            min_sizes: Vec::new(),
            expand: Vec::new(),
            weights: Vec::new(),
        }
    }
}
impl Layout for GridLayout {
    fn compute_min_size(&mut self, this: Id, ctx: &mut MinSizeContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            self.rows = 0;
            self.min_sizes.clear();
            ctx.set_this_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let len = children.len();
            self.rows = 1 + (len as u32 - 1) / self.columns;
            let columns = self.columns.min(children.len() as u32) as usize;
            let len = columns + self.rows as usize;
            self.min_sizes.resize(len, 0.0);
            self.expand.clear();
            self.expand.resize(len, false);
            self.weights.resize(len, 0.0);
            for (i, child) in children.into_iter().enumerate() {
                let rect = ctx.get_layouting(child);
                let col = i % columns;
                self.min_sizes[col] = self.min_sizes[col].max(rect.get_min_size()[0]);
                self.expand[col] |= rect.is_expand_x();
                self.weights[col] = rect.ratio_x;
                let row = columns + i / columns;
                self.min_sizes[row] = self.min_sizes[row].max(rect.get_min_size()[1]);
                self.expand[row] |= rect.is_expand_y();
                self.weights[row] = rect.ratio_y;
            }
            ctx.set_this_min_size([
                self.min_sizes[0..columns].iter().sum::<f32>()
                    + self.spacing[0] * self.columns.min(len as u32) as f32,
                self.min_sizes[columns..].iter().sum::<f32>()
                    + self.spacing[1] * (self.rows as usize - 1) as f32,
            ]);
        }
    }

    fn update_layouts(&mut self, this: Id, ctx: &mut LayoutContext) {
        let children = ctx.get_children(this);
        if children.is_empty() {
            return;
        }
        let columns = (self.columns as usize).min(children.len());
        let collumn_range = 0..columns;
        let row_range = columns..columns + self.rows as usize;
        let mut reserved_height = self.spacing[0] * columns as f32;
        let mut reserved_width = self.spacing[1] * self.rows as f32;
        let mut width_weight = 0.0;
        let mut height_weight = 0.0;
        for i in collumn_range.clone() {
            reserved_width += self.min_sizes[i];
            if self.expand[i] {
                width_weight += self.weights[i];
            }
        }
        for i in row_range.clone() {
            reserved_height += self.min_sizes[i];
            if self.expand[i] {
                height_weight += self.weights[i];
            }
        }
        let rect = ctx.get_layouting(this);
        let width = rect.get_width() - self.margins[0] - self.margins[2];
        let height = rect.get_height() - self.margins[1] - self.margins[3];
        let free_width = width - reserved_width;
        let free_height = height - reserved_height;
        let mut positions = vec![[0.0; 2]; self.columns as usize + self.rows as usize];
        let mut x = rect.get_rect()[0] + self.margins[0];
        if free_width <= 0.0 || width_weight == 0.0 {
            for i in collumn_range {
                positions[i][0] = x;
                positions[i][1] = x + self.min_sizes[i];
                x += self.spacing[0] + self.min_sizes[i];
            }
        } else {
            for i in collumn_range {
                if self.expand[i] {
                    // FIXME: this implementation imply that rects with the same ratio
                    // may not have the same size when expanded
                    let width = self.min_sizes[i] + free_width * self.weights[i] / width_weight;
                    positions[i][0] = x;
                    positions[i][1] = x + width;
                    x += self.spacing[0] + width;
                } else {
                    positions[i][0] = x;
                    positions[i][1] = x + self.min_sizes[i];
                    x += self.spacing[0] + self.min_sizes[i];
                }
            }
        }

        let mut y = rect.get_rect()[1] + self.margins[1];
        if free_height <= 0.0 || height_weight == 0.0 {
            for i in row_range {
                positions[i][0] = y;
                positions[i][1] = y + self.min_sizes[i];
                y += self.spacing[1] + self.min_sizes[i];
            }
        } else {
            for i in row_range {
                if self.expand[i] {
                    // FIXME: this implementation imply that rects with the same ratio
                    // may not have the same size when expanded
                    let height = self.min_sizes[i] + free_height * self.weights[i] / height_weight;
                    positions[i][0] = y;
                    positions[i][1] = y + height;
                    y += self.spacing[1] + height;
                } else {
                    positions[i][0] = y;
                    positions[i][1] = y + self.min_sizes[i];
                    y += self.spacing[1] + self.min_sizes[i];
                }
            }
        }
        for (i, child) in children.into_iter().enumerate() {
            let col = i % self.columns as usize;
            let row = self.columns as usize + i / self.columns as usize;
            let rect = [
                positions[col][0],
                positions[row][0],
                positions[col][1],
                positions[row][1],
            ];
            ctx.set_designed_rect(child, rect);
        }
    }
}
