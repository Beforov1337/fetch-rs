use crate::logo::{Logo, char_weight_utf8, strip_ansi};

pub struct PointCloud {
    pub px: Vec<f32>,
    pub py: Vec<f32>,
    pub pz: Vec<f32>,
    pub nx: Vec<f32>,
    pub ny: Vec<f32>,
    pub nz: Vec<f32>,
    pub pcolor: Vec<bool>, // true = inner, false = outer
}

impl PointCloud {
    pub fn build(logo: &Logo, size_scale: f32, config_depth: f32) -> Self {
        let sx = 0.07f32;
        let sy = 0.14f32;

        let clean_lines: Vec<String> = logo
            .lines
            .iter()
            .map(|l| strip_ansi(l).to_string())
            .collect();

        let logo_rows = clean_lines.len();
        let logo_cols = clean_lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);

        if logo_rows == 0 || logo_cols == 0 {
            return Self {
                px: Vec::new(),
                py: Vec::new(),
                pz: Vec::new(),
                nx: Vec::new(),
                ny: Vec::new(),
                nz: Vec::new(),
                pcolor: Vec::new(),
            };
        }

        let cx = (logo_cols as f32 - 1.0) * 0.5;
        let cy = (logo_rows as f32 - 1.0) * 0.5;

        let z_layers = (6.0 * size_scale).max(6.0) as usize;

        let mut hmap = vec![vec![0.0f32; logo_cols]; logo_rows];
        for (r, line) in clean_lines.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                if c < logo_cols {
                    hmap[r][c] = char_weight_utf8(ch);
                }
            }
        }

        let mut gnx = vec![vec![0.0f32; logo_cols]; logo_rows];
        let mut gny = vec![vec![0.0f32; logo_cols]; logo_rows];
        let mut gnz = vec![vec![1.0f32; logo_cols]; logo_rows];

        for r in 0..logo_rows {
            for c in 0..logo_cols {
                if hmap[r][c] <= 0.0 {
                    continue;
                }
                let mut dhdx = 0.0f32;
                if c > 0 && c < logo_cols - 1 {
                    dhdx = (hmap[r][c + 1] - hmap[r][c - 1]) * 0.5;
                } else if c == 0 && logo_cols > 1 {
                    dhdx = hmap[r][c + 1] - hmap[r][c];
                } else if c > 0 {
                    dhdx = hmap[r][c] - hmap[r][c - 1];
                }

                let mut dhdy = 0.0f32;
                if r > 0 && r < logo_rows - 1 {
                    dhdy = (hmap[r + 1][c] - hmap[r - 1][c]) * 0.5;
                } else if r == 0 && logo_rows > 1 {
                    dhdy = hmap[r + 1][c] - hmap[r][c];
                } else if r > 0 {
                    dhdy = hmap[r][c] - hmap[r - 1][c];
                }

                dhdx /= sx;
                dhdy /= sy;

                let nnx = -dhdx;
                let nny = dhdy;
                let nnz = 1.0f32;
                let len = (nnx * nnx + nny * nny + nnz * nnz).sqrt();
                gnx[r][c] = nnx / len;
                gny[r][c] = nny / len;
                gnz[r][c] = nnz / len;
            }
        }

        let zmax = 0.18f32 * config_depth;
        let subdiv = (size_scale as usize).max(1);

        let mut px = Vec::with_capacity(30000);
        let mut py = Vec::with_capacity(30000);
        let mut pz = Vec::with_capacity(30000);
        let mut nx = Vec::with_capacity(30000);
        let mut ny = Vec::with_capacity(30000);
        let mut nz = Vec::with_capacity(30000);
        let mut pcolor = Vec::with_capacity(30000);

        for row in 0..logo_rows {
            for col in 0..logo_cols {
                let h = hmap[row][col];
                if h <= 0.0 {
                    continue;
                }

                for sr in 0..subdiv {
                    for sc in 0..subdiv {
                        let frow = row as f32 + sr as f32 / subdiv as f32;
                        let fcol = col as f32 + sc as f32 / subdiv as f32;

                        let ox = (fcol - cx) * sx;
                        let oy = (cy - frow) * sy;
                        let zr = h * zmax;

                        // Edge check
                        let mut is_edge = false;
                        'edge_loop: for dr in -1i32..=1 {
                            for dc in -1i32..=1 {
                                if dr == 0 && dc == 0 {
                                    continue;
                                }
                                let nr = row as i32 + dr;
                                let nc = col as i32 + dc;
                                if nr >= 0
                                    && nr < logo_rows as i32
                                    && nc >= 0
                                    && nc < logo_cols as i32
                                {
                                    if hmap[nr as usize][nc as usize] <= 0.0 {
                                        is_edge = true;
                                        break 'edge_loop;
                                    }
                                } else {
                                    is_edge = true;
                                    break 'edge_loop;
                                }
                            }
                        }

                        let layers = if is_edge || h < 0.15 { 2 } else { z_layers };

                        for k in 0..layers {
                            let t = (k as f32 / (layers - 1).max(1) as f32) - 0.5;
                            px.push(ox);
                            py.push(oy);
                            pz.push(t * 2.0 * zr);

                            let is_face = k == 0 || k == layers - 1;
                            pcolor.push(is_face);

                            if k == 0 {
                                nx.push(gnx[row][col]);
                                ny.push(gny[row][col]);
                                nz.push(-gnz[row][col]);
                            } else if k == layers - 1 {
                                nx.push(gnx[row][col]);
                                ny.push(gny[row][col]);
                                nz.push(gnz[row][col]);
                            } else {
                                let mut ex = 0.0f32;
                                let mut ey = 0.0f32;
                                for dr in -1i32..=1 {
                                    for dc in -1i32..=1 {
                                        if dr == 0 && dc == 0 {
                                            continue;
                                        }
                                        let nr = row as i32 + dr;
                                        let nc = col as i32 + dc;
                                        if nr >= 0
                                            && nr < logo_rows as i32
                                            && nc >= 0
                                            && nc < logo_cols as i32
                                            && hmap[nr as usize][nc as usize] < h
                                        {
                                            ex += dc as f32;
                                            ey += (-dr) as f32;
                                        }
                                    }
                                }
                                let el = (ex * ex + ey * ey).sqrt();
                                if el > 1e-6 {
                                    ex /= el;
                                    ey /= el;
                                }
                                let tn = (k as f32 / (layers - 1) as f32) * 2.0 - 1.0;
                                let side = (1.0 - tn * tn).max(0.0).sqrt();
                                nx.push(ex * side);
                                ny.push(ey * side);
                                nz.push(tn);
                            }
                        }
                    }
                }
            }
        }

        Self {
            px,
            py,
            pz,
            nx,
            ny,
            nz,
            pcolor,
        }
    }
}

pub fn visible_width(s: &str) -> usize {
    let mut w = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && ((bytes[i] >= b'0' && bytes[i] <= b'9') || bytes[i] == b';') {
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // Final command byte
            }
            continue;
        }
        if (bytes[i] & 0xC0) != 0x80 {
            w += 1;
        }
        i += 1;
    }
    w
}

pub struct Renderer {
    pub anim_width: usize,
    pub render_height: usize,
    pub logo_height: usize,
    pub layout_stacked: bool,
    pub info_clip_cols: isize,
    pub fixed_y_center: f32,
    pub shading_ramp: Vec<char>,
    zbuf: Vec<f32>,
    lumbuf: Vec<f32>,
    colorbuf: Vec<bool>,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        term_cols: usize,
        term_rows: usize,
        info_lines_count: usize,
        info_max_cols: usize,
        shading_chars_opt: Option<&str>,
        custom_height: Option<usize>,
        size_scale: f32,
        point_cloud: &PointCloud,
    ) -> Self {
        let ramp: Vec<char> = shading_chars_opt
            .map(|s| s.chars().collect())
            .unwrap_or_else(|| ".,-~:;=!*#$@".chars().collect());

        let show_info = info_lines_count > 0;
        let mut rows = term_rows;
        if rows > 1 {
            rows -= 1; // 1 row margin to prevent scroll-jitter
        }

        let mut ideal = (36.0 * size_scale) as usize;
        if let Some(h) = custom_height {
            ideal = h;
        } else if show_info {
            let info_height = info_lines_count + 2;
            if ideal < info_height {
                ideal = info_height;
            }
        }
        ideal = ideal.clamp(20, 200);

        let mut render_height = ideal;
        let mut anim_width = 60usize;
        let mut layout_stacked = false;
        let mut info_clip_cols = -1isize;
        let gap = 2usize;

        if term_cols == 0 {
            if rows > 0 && render_height > rows {
                render_height = rows;
            }
        } else {
            let info_cols = if show_info { info_max_cols } else { 0 };
            if show_info && term_cols < 60 + gap + info_cols {
                anim_width = term_cols.saturating_sub(gap + info_cols);
                if anim_width < 20 {
                    layout_stacked = true;
                    anim_width = 60.min(term_cols);
                }
            } else if anim_width > term_cols {
                anim_width = term_cols;
            }

            let wcap = anim_width * 3 / 5;
            if layout_stacked {
                if render_height > wcap {
                    render_height = wcap;
                }
                if rows > 0 {
                    let budget = rows.saturating_sub(info_lines_count + 1);
                    if budget < render_height {
                        render_height = budget;
                    }
                    if render_height < 6 {
                        render_height = 0;
                    }
                }
                info_clip_cols = term_cols as isize;
            } else {
                if show_info {
                    info_clip_cols = (term_cols.saturating_sub(anim_width + gap)) as isize;
                }
                if rows > 0 && render_height > rows {
                    render_height = rows;
                }
            }
        }

        let wcap = anim_width * 3 / 5;
        let logo_height = if render_height > wcap {
            wcap
        } else {
            render_height
        };

        let k1 = 37.0f32 * (logo_height as f32 / 36.0) * size_scale;
        let k2 = 5.5f32;

        let mut face_up = 0.0f32;
        let mut face_dn = 0.0f32;
        for i in 0..point_cloud.px.len() {
            let zc = point_cloud.pz[i] + k2;
            if zc < 0.1 {
                continue;
            }
            let ys = k1 * point_cloud.py[i] / zc;
            if ys > face_up {
                face_up = ys;
            }
            if -ys > face_dn {
                face_dn = -ys;
            }
        }
        let fixed_y_center = face_up + 1.0;

        if custom_height.is_none() && !layout_stacked {
            let logo_bottom = (fixed_y_center + face_dn) as usize + 2;
            let fetch_start = if show_info { 1 } else { 0 };
            let info_bottom = if show_info {
                fetch_start + info_lines_count + 1
            } else {
                0
            };
            let needed = logo_bottom.max(info_bottom);
            if needed < render_height {
                render_height = needed;
            }
        }

        let total_cells = render_height * anim_width;

        Self {
            anim_width,
            render_height,
            logo_height,
            layout_stacked,
            info_clip_cols,
            fixed_y_center,
            shading_ramp: ramp,
            zbuf: vec![0.0; total_cells],
            lumbuf: vec![0.0; total_cells],
            colorbuf: vec![false; total_cells],
        }
    }

    pub fn clear(&mut self) {
        self.zbuf.fill(0.0);
        self.lumbuf.fill(0.0);
        self.colorbuf.fill(false);
    }

    pub fn render_frame(
        &mut self,
        cloud: &PointCloud,
        angle_a: f32,
        angle_b: f32,
        k1: f32,
        k2: f32,
    ) {
        self.clear();

        let ca = angle_a.cos();
        let sa = angle_a.sin();
        let cb = angle_b.cos();
        let sb = angle_b.sin();

        // Light direction (front-top-left)
        let lx = 0.577f32;
        let ly = 0.577f32;
        let lz = 0.577f32;

        let half_aw = self.anim_width as f32 * 0.5;
        let y_center = self.fixed_y_center;
        let k1x2 = k1 * 2.0;

        for i in 0..cloud.px.len() {
            let px = cloud.px[i];
            let py = cloud.py[i];
            let pz = cloud.pz[i];

            let nx = cloud.nx[i];
            let ny = cloud.ny[i];
            let nz = cloud.nz[i];

            // Rotation
            let y1 = py * ca - pz * sa;
            let z1 = py * sa + pz * ca;
            let x2 = px * cb + z1 * sb;
            let z2 = -px * sb + z1 * cb;
            let y2 = y1;

            let ny1 = ny * ca - nz * sa;
            let nz1 = ny * sa + nz * ca;
            let nx2 = nx * cb + nz1 * sb;
            let nz2 = -nx * sb + nz1 * cb;
            let ny2 = ny1;

            let zc = z2 + k2;
            if zc < 0.1 {
                continue;
            }

            let ooz = 1.0 / zc;
            let xs = (half_aw + k1x2 * x2 * ooz) as isize;
            let ys = (y_center - k1 * y2 * ooz) as isize;

            if xs >= 0
                && (xs as usize) < self.anim_width
                && ys >= 0
                && (ys as usize) < self.render_height
            {
                let idx = ys as usize * self.anim_width + xs as usize;
                if ooz > self.zbuf[idx] {
                    let mut diff = nx2 * lx + ny2 * ly + nz2 * lz;
                    if diff < 0.0 {
                        diff = 0.0;
                    }

                    let spec_dot = (nx2 * 0.3 + ny2 * 0.3 + nz2 * 0.9).max(0.0);
                    let spec = spec_dot.powi(8);

                    let l = (0.10 + 0.60 * diff + 0.30 * spec).min(1.0);

                    self.zbuf[idx] = ooz;
                    self.lumbuf[idx] = l;
                    self.colorbuf[idx] = cloud.pcolor[i];
                }
            }
        }
    }

    pub fn draw_to_buffer(
        &self,
        buf: &mut String,
        logo: &Logo,
        info: Option<&crate::info::SysInfo>,
        use_color: bool,
        _box_border: bool,
    ) {
        use std::fmt::Write;

        let smax = self.shading_ramp.len().saturating_sub(1);
        let reset = "\x1b[0m";

        for row in 0..self.render_height {
            let mut prev_color: Option<bool> = None;

            for col in 0..self.anim_width {
                let idx = row * self.anim_width + col;
                let z = self.zbuf[idx];

                if z <= 0.0 {
                    if prev_color.is_some() && use_color {
                        buf.push_str(reset);
                        prev_color = None;
                    }
                    buf.push(' ');
                } else {
                    let l = self.lumbuf[idx];
                    let ci = ((l * smax as f32).round() as usize).min(smax);
                    let ch = self.shading_ramp[ci];
                    let is_inner = self.colorbuf[idx];

                    if use_color && prev_color != Some(is_inner) {
                        let color_code = if is_inner {
                            &logo.color_inner
                        } else {
                            &logo.color_outer
                        };
                        buf.push_str(color_code);
                        prev_color = Some(is_inner);
                    }
                    buf.push(ch);
                }
            }

            if prev_color.is_some() && use_color {
                buf.push_str(reset);
            }

            // Info rendering beside logo
            if !self.layout_stacked
                && let Some(inf) = info
            {
                let start_row = 1;
                if row >= start_row {
                    let fi = row - start_row;
                    if fi == 0 {
                        buf.push_str("  ");
                        buf.push_str(&inf.title);
                    } else if fi == 1 {
                        buf.push_str("  ");
                        buf.push_str(&"-".repeat(inf.title_len));
                    } else if fi - 2 < inf.lines.len() {
                        let (label, val) = &inf.lines[fi - 2];
                        buf.push_str("  ");
                        let _ = write!(buf, "\x1b[1;34m{}\x1b[0m: {}", label, val);
                    }
                }
            }

            buf.push_str("\x1b[K\r\n");
        }

        // Stacked info layout if narrow terminal
        if self.layout_stacked
            && let Some(inf) = info
        {
            buf.push_str("\r\n");
            let _ = write!(buf, "{}\r\n", inf.title);
            let _ = write!(buf, "{}\r\n", "-".repeat(inf.title_len));
            for (label, val) in &inf.lines {
                let _ = write!(buf, "\x1b[1;34m{:>10}\x1b[0m: {}\r\n", label, val);
            }
        }
    }
}
