use std::io::{Write, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size as term_size},
};

use fetch_rs::cli::Cli;
use fetch_rs::info::SysInfo;
use fetch_rs::logo::Logo;
use fetch_rs::render::{PointCloud, Renderer};

fn main() -> Result<()> {
    let args = Cli::parse();

    let logo = Logo::load(args.logo.as_deref());
    let mut sys_info = if !args.no_info {
        Some(SysInfo::gather())
    } else {
        None
    };

    let (t_cols, t_rows) = term_size().unwrap_or((80, 24));
    let info_len = sys_info.as_ref().map(|s| s.lines.len() + 2).unwrap_or(0);
    let info_max_cols = sys_info.as_ref().map(|s| s.max_line_width()).unwrap_or(0);

    let point_cloud = PointCloud::build(&logo, args.size, args.depth);

    let mut renderer = Renderer::new(
        t_cols as usize,
        t_rows as usize,
        info_len,
        info_max_cols,
        args.shading_chars.as_deref(),
        args.height,
        args.size,
        &point_cloud,
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_setup(r);

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, Hide)?;
    // Clear screen and position at top
    out.write_all(b"\x1b[2J\x1b[H")?;
    out.flush()?;

    let mut angle_a = 0.0f32;
    let mut angle_b = 0.0f32;
    let mut drag_vx = 0.0f32;
    let mut drag_vy = 0.0f32;
    let mut dragging = false;
    let mut last_mouse_pos = (0u16, 0u16);

    let rotate_x = !args.rotate_y;
    let rotate_y = !args.rotate_x;
    let speed = args.speed;

    let k1 = 37.0f32 * (renderer.logo_height as f32 / 36.0) * args.size;
    let k2 = 5.5f32;

    let frame_duration = Duration::from_millis(33); // ~30 fps
    let mut frame_count = 0;
    let mut output_buffer = String::with_capacity(32768);

    while running.load(Ordering::Relaxed) {
        if !args.infinite && args.frames > 0 && frame_count >= args.frames {
            break;
        }

        let frame_start = Instant::now();

        // Handle events
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('c')
                        && key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        running.store(false, Ordering::Relaxed);
                        break;
                    } else {
                        // Any keypress terminates fetch (just like C version)
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        dragging = true;
                        last_mouse_pos = (mouse_event.column, mouse_event.row);
                        drag_vx = 0.0;
                        drag_vy = 0.0;
                    }
                    MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                        if dragging {
                            let dx = mouse_event.column as i32 - last_mouse_pos.0 as i32;
                            let dy = mouse_event.row as i32 - last_mouse_pos.1 as i32;
                            drag_vy = -dx as f32 * 0.03;
                            drag_vx = -dy as f32 * 0.03;
                            angle_b += drag_vy;
                            angle_a += drag_vx;
                            last_mouse_pos = (mouse_event.column, mouse_event.row);
                        }
                    }
                    MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                        dragging = false;
                    }
                    _ => {}
                },
                Event::Resize(cols, rows) => {
                    renderer = Renderer::new(
                        cols as usize,
                        rows as usize,
                        info_len,
                        info_max_cols,
                        args.shading_chars.as_deref(),
                        args.height,
                        args.size,
                        &point_cloud,
                    );
                    execute!(out, Clear(ClearType::All))?;
                }
                _ => {}
            }
        }

        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Periodic info refresh (~1 second = 30 frames)
        if frame_count % 30 == 0
            && let Some(inf) = &mut sys_info
        {
            inf.refresh_dynamic();
        }

        // Physics / rotation update
        if !dragging {
            if drag_vx != 0.0 || drag_vy != 0.0 {
                angle_a += drag_vx;
                angle_b += drag_vy;
                drag_vx *= 0.95;
                drag_vy *= 0.95;
                if drag_vx.abs() < 0.001 {
                    drag_vx = 0.0;
                }
                if drag_vy.abs() < 0.001 {
                    drag_vy = 0.0;
                }
            } else {
                if rotate_x {
                    angle_a += 0.04 * speed;
                }
                if rotate_y {
                    angle_b += 0.06 * speed;
                }
            }
        }

        // Render 3D point cloud into buffers
        renderer.render_frame(&point_cloud, angle_a, angle_b, k1, k2);

        // Draw buffer to terminal string
        output_buffer.clear();
        output_buffer.push_str("\x1b[H"); // Cursor to home (0,0)
        renderer.draw_to_buffer(
            &mut output_buffer,
            &logo,
            sys_info.as_ref(),
            !args.no_color,
            args.box_border,
        );

        out.write_all(output_buffer.as_bytes())?;
        out.flush()?;

        frame_count += 1;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // Cleanup terminal
    execute!(out, Show)?;
    disable_raw_mode()?;
    let final_y = renderer.render_height + 1;
    print!("\x1b[{final_y};1H\r\n");
    let _ = stdout().flush();

    Ok(())
}

fn ctrlc_setup(running: Arc<AtomicBool>) {
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, running.clone());
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, running);
}
