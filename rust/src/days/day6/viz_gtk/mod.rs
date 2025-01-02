use std::sync::atomic::Ordering;

use anyhow::Result;
use gtk::{cairo, gdk, glib, prelude::*};

use super::{Cell, Direction, Map, Pos};

use self::utils::eat_err;

mod state;
mod utils;

const APP_ID: &str = "com.nelsonearle.adventofcode.day6.viz";

pub(super) fn viz_main(map: &mut Map, pos: Pos, direction: Direction) {
    eat_err(unsafe { viz_main_imp(map, pos, direction) });
}

unsafe fn viz_main_imp(map: &mut Map, pos: Pos, direction: Direction) -> Result<()> {
    // NOTE: IMPORTANT: Initialize the whole struct
    *state::APP_STATE.write().unwrap() = state::AppState_ {
        map: map.clone(),
        pos,
        direction,
        probe_succeeded: false,
    };

    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(cb_activate);

    let exit_code = app.run_with_args(&[] as &[&str]);
    if exit_code != glib::ExitCode::SUCCESS {
        eprintln!("{exit_code:?}");
    }

    Ok(())
}

fn cb_activate(app: &gtk::Application) {
    // CSS

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(
        r"
        box { background-color: #000000; }
        position-label { color: white; }
    ",
    );

    let display = gdk::Display::default().unwrap();
    gtk::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Drawing Area

    let drawing_area = gtk::DrawingArea::builder()
        .content_width(800)
        .content_height(800)
        .build();

    // Cell Highlight

    // let cell_highlight = state::CellHighlight::new();

    // let cell_hl_draw_action = gio::ActionEntry::builder("draw")
    //     .activate(glib::clone!(
    //         #[weak]
    //         drawing_area,
    //         move |_, _, _| {
    //             // let ctx = drawing_area.pango_context();
    //             // ctx.rectangle(100.0, 100.0, 50.0, 50.0);
    //             let ctx = cairo::Context::default();
    //         }
    //     ))
    //     .build();
    // let cell_hl_actions = gio::SimpleActionGroup::new();
    // cell_hl_actions.add_action_entries([cell_hl_draw_action]);
    // // cell_highlight.insert_action_group("cell-highlight", Some(&cell_hl_actions));
    // drawing_area.insert_action_group("cell-highlight", Some(&cell_hl_actions));

    // Position Label

    let position_label = gtk::Label::builder()
        .css_name("position-label")
        .label("abc")
        .build();

    // Layout

    let layout_box = gtk::Box::builder()
        .css_name("box")
        .orientation(gtk::Orientation::Vertical)
        .build();
    layout_box.append(&drawing_area);
    // layout_box.append(&cell_highlight);
    layout_box.append(&position_label);

    // Window

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("AoC - Day 6 - Part 2")
        .default_width(800)
        .default_height(800)
        .resizable(false)
        .child(&layout_box)
        .build();

    // Draw

    drawing_area.set_draw_func(glib::clone!(move |widget, ctx, w, h| eat_err(draw(
        widget, ctx, w, h
    ))));

    // Update Position Label

    fn get_mouse_position(window: gtk::ApplicationWindow) -> Option<(f64, f64, gdk::ModifierType)> {
        let display = gdk::Display::default().unwrap();
        let pointer = display.default_seat().unwrap().pointer().unwrap();
        let surface = window.root().unwrap().surface().unwrap();
        surface.device_position(&pointer)
    }

    glib::timeout_add_local(
        std::time::Duration::from_millis(20),
        glib::clone!(
            #[weak]
            window,
            #[weak]
            drawing_area,
            #[weak]
            position_label,
            #[upgrade_or]
            glib::ControlFlow::Continue,
            move || {
                match get_mouse_position(window) {
                    Some((x, y, _)) => {
                        // Set label
                        let row = (y / 6.0).trunc() as usize;
                        let col = (x / 6.0).trunc() as usize;
                        position_label.set_text(&format!("row={row} col={col}"));
                        // Draw cell highlight
                        {
                            let mut pointer_loc = state::POINTER_POSITION.write().unwrap();
                            *pointer_loc = Pos::new(row, col);
                        }
                        drawing_area.queue_draw();
                        // drawing_area
                        //     .activate_action("cell-highlight.draw", None)
                        //     .unwrap();
                    }
                    None => position_label.set_text("row= col="),
                }
                glib::ControlFlow::Continue
            }
        ),
    );

    // Step Solver

    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(glib::clone!(
        #[weak]
        app,
        #[weak]
        drawing_area,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |controller, keyval, keycode, modifier| {
            cb_key_pressed(app, drawing_area, controller, keyval, keycode, modifier)
        }
    ));
    window.add_controller(key_controller);

    window.present();
}

fn draw(_widget: &gtk::DrawingArea, ctx: &cairo::Context, width: i32, height: i32) -> Result<()> {
    let state = state::APP_STATE.read().unwrap();

    let map_width = state.map.width as f64;
    let map_height = state.map.height as f64;

    let cell_size = {
        let cell_w = (width as f64 / map_width).trunc();
        let cell_h = (height as f64 / map_height).trunc();
        cell_w.min(cell_h)
    };
    assert_eq!(
        cell_size, 6.0,
        "other parts of this app hard-code the cell size in calculationsp"
    );

    let color_map_bg = gdk::RGBA::new(0.2, 0.2, 0.2, 1.);
    let color_cell_obstacle = gdk::RGBA::new(1., 0., 0., 1.);
    let color_cell_target = gdk::RGBA::new(1., 0., 1., 0.7);
    let color_cell_cursor = gdk::RGBA::new(1., 1., 1., 1.);
    let color_path = gdk::RGBA::new(0.5, 0.5, 0.5, 1.);
    let color_path_probe_failure = gdk::RGBA::new(0.8, 0.2, 0.2, 0.3);
    let color_path_probe_success = gdk::RGBA::new(0.2, 0.8, 0.2, 0.3);
    let color_pointer_cell_highlight = gdk::RGBA::new(1.0, 1.0, 1.0, 1.0);

    let _did_map_change = state::DID_MAP_CHANGE
        .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
        .is_ok();

    ctx.set_source_color(&color_map_bg);
    ctx.rectangle(0.0, 0.0, map_width * cell_size, map_height * cell_size);
    ctx.fill()?;

    // Draw obstacles

    for (r, row) in state.map.grid.chunks(state.map.width).enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let x = c as f64 * cell_size;
            let y = r as f64 * cell_size;

            if *cell == Cell::Obstacle {
                draw_cell(ctx, x, y, cell_size, &color_cell_obstacle)?;
            }
        }
    }

    let to_cell_center = |pos: Pos| {
        (
            (pos.col as f64 + 0.5) * cell_size,
            (pos.row as f64 + 0.5) * cell_size,
        )
    };
    // let to_col_row = |x: f64, y: f64| {
    //     Pos::new(
    //         (y / cell_size).trunc() as usize,
    //         (x / cell_size).trunc() as usize,
    //     )
    // };

    // Draw current walk path

    {
        ctx.set_source_color(&color_path);
        ctx.set_line_width(cell_size * 0.8);
        ctx.set_line_cap(cairo::LineCap::Square);

        let mut direction = state
            .map
            ._viz_walk_path
            .first()
            .map(|p| p.1.rotate())
            .unwrap_or(Direction::South);

        // if _did_map_change {
        //     println!("WALK:");
        //     for (c, d) in &state.map._walk_path {
        //         print!(" / {c} {d}");
        //     }
        //     println!();
        // }

        for &(walk_pos, walk_dir) in &state.map._viz_walk_path {
            // if _did_map_change {
            //     println!("walk point {walk_pos} {walk_dir}");
            // }

            if walk_dir == direction {
                continue;
            }

            direction = walk_dir;

            let (x, y) = to_cell_center(walk_pos);

            // if _did_map_change {
            //     let current = ctx.current_point()?;
            //     let current = to_col_row(current.0, current.1);
            //     let next = to_col_row(x, y);
            //     println!("walk line segment | {} -> {}", current, next);
            // }

            ctx.line_to(x, y);
        }

        if let Some((last_pos, _last_dir)) = state.map._viz_walk_path.last().copied() {
            let (x, y) = to_cell_center(last_pos);

            // if _did_map_change {
            //     let current = ctx.current_point()?;
            //     let current = to_col_row(current.0, current.1);
            //     let next = to_col_row(x, y);
            //     println!("walk line segment | {} -> {}", current, next);
            // }

            ctx.line_to(x, y);
        }

        ctx.stroke()?;
    }

    // Draw current probe path

    {
        if state.probe_succeeded {
            ctx.set_source_color(&color_path_probe_success);
        } else {
            ctx.set_source_color(&color_path_probe_failure);
        }
        ctx.set_line_width(cell_size * 0.8);
        ctx.set_line_cap(cairo::LineCap::Square);

        let mut direction = state
            .map
            ._viz_probe_path
            .first()
            .map(|p| p.1.rotate())
            .unwrap_or(Direction::South);

        for &(probe_pos, probe_dir) in &state.map._viz_probe_path {
            // if _did_map_change {
            //     print!(" => {probe_pos} {probe_dir}");
            // }

            if probe_dir == direction {
                continue;
            }

            direction = probe_dir;

            let (x, y) = to_cell_center(probe_pos);

            // if _did_map_change {
            //     let current = ctx.current_point()?;
            //     let current = to_col_row(current.0, current.1);
            //     let next = to_col_row(x, y);
            //     println!("probe line segment | {} -> {}", current, next);
            // }

            ctx.line_to(x, y);
        }

        // if _did_map_change {
        //     println!();
        // }

        if let Some((last_pos, _last_dir)) = state.map._viz_probe_path.last().copied() {
            let (x, y) = to_cell_center(last_pos);

            // if _did_map_change {
            //     let current = ctx.current_point()?;
            //     let current = to_col_row(current.0, current.1);
            //     let next = to_col_row(x, y);
            //     println!("probe line segment | {} -> {}", current, next);
            // }

            ctx.line_to(x, y);
        }

        ctx.stroke()?;
    }

    // Draw obstacle candidate

    {
        let x = state.map._viz_obstacle.col as f64 * cell_size;
        let y = state.map._viz_obstacle.row as f64 * cell_size;
        draw_cell(ctx, x, y, cell_size, &color_cell_target)?;
    }

    // Draw cursor

    {
        if let Some((last_pos, _last_dir)) = state.map._viz_walk_path.last().copied() {
            let x = last_pos.col as f64 * cell_size;
            let y = last_pos.row as f64 * cell_size;
            draw_cursor(ctx, x, y, cell_size, &color_cell_cursor)?;
        }
    }

    // Draw pointer cell highlight

    {
        let pointer_loc = *state::POINTER_POSITION.read().unwrap();
        let (x, y) = (
            pointer_loc.col as f64 * cell_size,
            pointer_loc.row as f64 * cell_size,
        );
        ctx.rectangle(x, y, cell_size, cell_size);
        ctx.set_source_color(&color_pointer_cell_highlight);
        ctx.set_line_width(1.0);
        ctx.stroke()?;
    }

    Ok(())
}

fn draw_cell(ctx: &cairo::Context, x: f64, y: f64, size: f64, color: &gdk::RGBA) -> Result<()> {
    ctx.rectangle(x, y, size, size);
    ctx.set_source_color(color);
    ctx.fill()?;
    Ok(())
}

/// [Source](https://stackoverflow.com/a/11373119)
fn draw_cursor(ctx: &cairo::Context, x: f64, y: f64, size: f64, color: &gdk::RGBA) -> Result<()> {
    ctx.save()?;

    ctx.rectangle(x, y, size, size);
    ctx.set_source_color(color);
    ctx.fill()?;

    let glow_x = x + size / 2.;
    let glow_y = y + size / 2.;

    let glow_inner_r = 0.;
    let glow_outer_r = size * 3.;

    let p = cairo::RadialGradient::new(glow_x, glow_y, glow_inner_r, glow_x, glow_y, glow_outer_r);
    p.add_color_stop_rgba(0., 0.7, 0.7, 0.7, 1.);
    p.add_color_stop_rgba(1., 0.7, 0.7, 0.7, 0.);

    ctx.rectangle(
        glow_x - glow_outer_r,
        glow_y - glow_outer_r,
        2.0 * glow_outer_r,
        2.0 * glow_outer_r,
    );
    ctx.clip();
    ctx.set_source(&p)?;
    ctx.mask(p)?;

    ctx.restore()?;

    Ok(())
}

fn cb_key_pressed(
    app: gtk::Application,
    drawing_area: gtk::DrawingArea,
    _controller: &gtk::EventControllerKey,
    keyval: gdk::Key,
    _keycode: u32,
    modifier: gdk::ModifierType,
) -> glib::Propagation {
    // eprintln!("Press {:?}", utils::K_(keyval, modifier));

    if modifier == gdk::ModifierType::META_MASK && keyval == gdk::Key::q {
        app.quit();
    } else if keyval == gdk::Key::space {
        let mut state = state::APP_STATE.write().unwrap();
        {
            let state::AppState_ {
                map,
                pos,
                direction,
                probe_succeeded,
            } = &mut *state;
            let mut path = state::PATH.write().unwrap();
            // map.viz_run_to_obstacle(cursor, direction);
            *probe_succeeded = map.viz_walk_and_find_loop_candidates(&mut path, pos, direction);
        }

        state::DID_MAP_CHANGE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .unwrap();

        drawing_area.queue_draw();
    }

    glib::Propagation::Proceed
}
