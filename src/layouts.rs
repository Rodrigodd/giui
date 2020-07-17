use crate::{Id, Layout, Rect, Widgets};

pub struct VBoxLayout {
    spacing: f32,
    margins: [f32; 4],
    aling: i8,
}
impl VBoxLayout {
    pub fn new(spacing: f32, margins: [f32; 4], aling: i8) -> Self {
        Self {
            spacing,
            margins,
            aling,
        }
    }
}
impl Layout for VBoxLayout {
    fn compute_min_size(&mut self, this: Id, widgets: &mut Widgets) {
        let children = widgets.get_children(this);
        if children.is_empty() {
            widgets.get_rect(this).set_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let mut min_width: f32 = 0.0;
            let mut min_height: f32 =
                self.margins[1] + self.margins[3] + (children.len() - 1) as f32 * self.spacing;
            for child in children {
                let [width, height] = widgets.get_rect(child).min_size;
                min_width = min_width.max(width);
                min_height += height;
            }
            widgets
                .get_rect(this)
                .set_min_size([min_width + self.margins[0] + self.margins[2], min_height]);
            println!("min_size: {:?}", widgets.get_rect(this).min_size);
        }
    }

    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets) {
        let children = widgets.get_children(this);
        if children.is_empty() {
            return;
        }
        let mut reserved_height = self.spacing * (children.len() - 1) as f32;
        let mut max_weight = 0.0;
        for child in children {
            let rect = widgets.get_rect(child);
            reserved_height += rect.min_size[1];
            if rect.is_expand_y() {
                max_weight += rect.ratio_y;
            }
        }
        let rect = widgets.get_rect(this);
        let left = rect.rect[0] + self.margins[0];
        let right = rect.rect[2] - self.margins[2];
        let mut y = rect.rect[1] + self.margins[1];
        let height = rect.get_height() - self.margins[1] - self.margins[3];
        let free_height = height - reserved_height;
        if free_height <= 0.0 || max_weight == 0.0 {
            match self.aling {
                0 => y += free_height / 2.0,
                1 => y += free_height,
                _ => {}
            }
            for child in widgets.get_children(this) {
                let rect = widgets.get_rect(child);
                rect.set_designed_rect([left, y, right, y + rect.min_size[1]]);
                y += self.spacing + rect.min_size[1];
            }
        } else {
            for child in widgets.get_children(this) {
                let rect = widgets.get_rect(child);
                if rect.is_expand_y() {
                    // FIXME: this implementation imply that rect with same ratio,
                    // may not have the same size when expanded
                    let height = rect.min_size[1] + free_height * rect.ratio_y / max_weight;
                    rect.set_designed_rect([left, y, right, y + height]);
                    y += self.spacing + height;
                } else {
                    rect.set_designed_rect([left, y, right, y + rect.min_size[1]]);
                    y += self.spacing + rect.min_size[1];
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
    fn compute_min_size(&mut self, this: Id, widgets: &mut Widgets) {
        let children = widgets.get_children(this);
        if children.is_empty() {
            self.rows = 0;
            self.min_sizes.clear();
            widgets.get_rect(this).set_min_size([
                self.margins[0] + self.margins[2],
                self.margins[1] + self.margins[3],
            ]);
        } else {
            let len = children.len();
            self.rows = (len as u32 - 1) / self.columns + 1;
            self.min_sizes
                .resize(self.columns as usize + self.rows as usize, 0.0);
            self.expand.clear();
            self.expand
                .resize(self.columns as usize + self.rows as usize, false);
            self.weights
                .resize(self.columns as usize + self.rows as usize, 0.0);
            for (i, child) in children.into_iter().enumerate() {
                let rect = widgets.get_rect(child);
                let col = i % self.columns as usize;
                self.min_sizes[col] = self.min_sizes[col].max(rect.min_size[0]);
                self.expand[col] |= rect.is_expand_x();
                self.weights[col] = rect.ratio_x;
                let row = self.columns as usize + i / self.columns as usize;
                self.min_sizes[row] = self.min_sizes[row].max(rect.min_size[1]);
                self.expand[row] |= rect.is_expand_y();
                self.weights[row] = rect.ratio_y;
            }
            widgets.get_rect(this).set_min_size([
                self.min_sizes[0..self.columns as usize].iter().sum::<f32>()
                    + self.spacing[0] * self.columns.min(len as u32) as f32,
                self.min_sizes[self.columns as usize..].iter().sum::<f32>()
                    + self.spacing[1] * (self.rows as usize - 1) as f32,
            ]);
        }
    }

    fn update_layouts(&mut self, this: Id, widgets: &mut Widgets) {
        let children = widgets.get_children(this);
        if children.is_empty() {
            return;
        }
        let collumn_range = 0..self.columns as usize;
        let row_range = self.columns as usize..self.columns as usize + self.rows as usize;
        let mut reserved_height = self.spacing[0] * self.columns.min(children.len() as u32) as f32;
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
        let rect = widgets.get_rect(this);
        let width = rect.get_width() - self.margins[0] - self.margins[2];
        let height = rect.get_height() - self.margins[1] - self.margins[3];
        let free_width = width - reserved_width;
        let free_height = height - reserved_height;
        let mut positions = vec![[0.0; 2]; self.columns as usize + self.rows as usize];
        println!("free_width: {} - {} = {}", width, reserved_width, free_width);
        println!("free_height: {} - {} = {}", height , reserved_height, free_height);
        let mut x = rect.rect[0] + self.margins[0];
        if free_width <= 0.0 || width_weight == 0.0 {
            for i in collumn_range {
                positions[i][0] = x;
                positions[i][1] = x + self.min_sizes[i];
                x += self.spacing[0] + self.min_sizes[i];
            }
        } else {
            println!("Expand X!");
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

        let mut y = rect.rect[1] + self.margins[1];
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
        println!("positions: {:?}", positions);
        for (i, child) in children.into_iter().enumerate() {
            let col = i % self.columns as usize;
            let row = self.columns as usize + i / self.columns as usize;
            let rect = [
                positions[col][0],
                positions[row][0],
                positions[col][1],
                positions[row][1],
            ];
            println!("designed_rect: {:?}", rect);
            widgets.get_rect(child).set_designed_rect(rect);
        }
    }
}
