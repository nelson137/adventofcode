use std::{cmp, ops, sync};

inventory::submit!(
    crate::days::DayModule::new(2025, 5)
        .with_executors(
            crate::day_part_executors![part1],
            crate::day_part_executors![part2],
        )
        .with_pt1_visualizer(part1_viz)
        .with_pt2_visualizer(part2_viz)
);

struct IngredientDatabase<It> {
    fresh_ingredient_id_range_buckets: Vec<IdRangeBucket>,
    available_ingredient_ids_iter: It,
}

struct IdRangeBucket {
    ranges: Vec<ops::RangeInclusive<u64>>,
    max: u64,
}

impl IdRangeBucket {
    fn new(range: ops::RangeInclusive<u64>) -> Self {
        let max = *range.end();
        Self {
            ranges: vec![range],
            max,
        }
    }
}

fn parse_ingredient_database(input: &str) -> IngredientDatabase<impl Iterator<Item = &str>> {
    let mut iter = input.lines();
    let mut fresh_ingredient_id_ranges = Vec::new();

    loop {
        let Some(line) = iter.next() else {
            break;
        };
        if line.is_empty() {
            break;
        }

        let (min, max) = line.split_once('-').unwrap();
        let range = min.parse::<u64>().unwrap()..=max.parse().unwrap();
        fresh_ingredient_id_ranges.push(range);
    }

    // Sort by range start asc.
    fresh_ingredient_id_ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()));

    let mut fresh_ingredient_id_range_buckets = Vec::<IdRangeBucket>::new();

    'ranges: for range in &fresh_ingredient_id_ranges {
        for bucket in &mut fresh_ingredient_id_range_buckets {
            if *range.start() > bucket.max {
                bucket.ranges.push(range.clone());
                bucket.max = cmp::max(bucket.max, *range.end());
                continue 'ranges;
            }
        }
        fresh_ingredient_id_range_buckets.push(IdRangeBucket::new(range.clone()));
    }

    IngredientDatabase {
        fresh_ingredient_id_range_buckets,
        available_ingredient_ids_iter: iter,
    }
}

impl<'input, It> IngredientDatabase<It>
where
    It: Iterator<Item = &'input str>,
{
    fn count_fresh_ids(&mut self) -> usize {
        let mut count = 0_usize;

        for line in &mut self.available_ingredient_ids_iter {
            let id: u64 = line.parse().unwrap();

            if self.fresh_ingredient_id_range_buckets.iter().any(move |b| {
                b.ranges
                    .binary_search_by(move |r| match () {
                        _ if r.contains(&id) => cmp::Ordering::Equal,
                        _ if *r.end() < id => cmp::Ordering::Less,
                        _ => cmp::Ordering::Greater,
                    })
                    .is_ok()
            }) {
                count += 1;
            }
        }

        count
    }
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut db = parse_ingredient_database(input);

    let fresh_ingredient_id_count = db.count_fresh_ids();

    Some(Box::new(fresh_ingredient_id_count))
}

const APP_ID: &str = "com.nelsonearle.adventofcode.year2025.day5.viz";

fn part1_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    use gtk::{gdk, glib, prelude::*};

    let db = parse_ingredient_database(input);

    let id_ranges: &[ops::RangeInclusive<u64>] = db
        .fresh_ingredient_id_range_buckets
        .iter()
        .flat_map(|b| &b.ranges)
        .cloned()
        .collect::<Vec<_>>()
        .leak();

    const SCALE: u64 = 1000_u64.pow(4) / 2;
    let max_range = id_ranges.iter().map(|r| *r.end() / SCALE).max().unwrap();

    let width = max_range as i32;
    let height = id_ranges.len() as i32 * 3;

    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| {
        let drawing_area = gtk::DrawingArea::builder()
            .content_width(width)
            .content_height(height)
            .build();

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("AoC - Year 2025 - Day 6 - Part 1")
            .default_width(width)
            .default_height(height)
            .resizable(false)
            .child(&drawing_area)
            .build();

        drawing_area.set_draw_func(move |_widget, ctx, _w, _h| {
            // const RED: gdk::RGBA = gdk::RGBA::new(1., 0., 0., 1.);
            const GREEN: gdk::RGBA = gdk::RGBA::new(0., 1., 0., 1.);
            // const BLUE: gdk::RGBA = gdk::RGBA::new(0., 0., 1., 1.);

            for (i, range) in id_ranges.iter().enumerate() {
                let start = *range.start() / SCALE;
                let end = *range.end() / SCALE;
                let extent = end - start;
                ctx.set_source_color(&GREEN);
                ctx.rectangle(start as f64, (i * 3) as f64, extent as f64, 2.);
                ctx.fill().unwrap();
            }
        });

        window.present();
    });

    let exit_code = app.run_with_args(&[] as &[&str]);
    if exit_code != glib::ExitCode::SUCCESS {
        eprintln!("{exit_code:?}");
    }

    None
}

const TRIM: usize = 164; // equal at 165

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut fresh_ingredient_id_ranges = Vec::new();

    for line in input.lines() {
        if line.is_empty() {
            break;
        }

        let (min, max) = line.split_once('-').unwrap();
        let start = min.parse::<u64>().unwrap();
        let end = max.parse().unwrap();
        fresh_ingredient_id_ranges.push(start..=end);
    }

    // Sort by range start asc.
    fresh_ingredient_id_ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()));

    fresh_ingredient_id_ranges
        .drain(fresh_ingredient_id_ranges.len() - TRIM..)
        .count();

    loop {
        let mut has_overlapping = false;

        for i in (0..fresh_ingredient_id_ranges.len()).rev() {
            let r = &fresh_ingredient_id_ranges[i];
            let (r_start, r_end) = (*r.start(), *r.end());
            if r_start == u64::MAX {
                continue;
            }

            for j in 0..fresh_ingredient_id_ranges.len() {
                if j == i {
                    continue;
                }
                let range = &mut fresh_ingredient_id_ranges[j];
                if *range.start() == u64::MAX {
                    continue;
                }
                if range.contains(&r_start) || range.contains(&r_end) {
                    has_overlapping = true;
                    let start = cmp::min(*range.start(), r_start);
                    let end = cmp::max(*range.end(), r_end);
                    *range = start..=end;
                    fresh_ingredient_id_ranges[i] = u64::MAX..=u64::MAX;
                    break;
                }
            }
        }

        if !has_overlapping {
            break;
        }
    }

    let count = fresh_ingredient_id_ranges
        .iter()
        .filter(|r| *r.start() != u64::MAX)
        .map(|r| *r.end() - *r.start() + 1)
        .sum::<u64>();

    Some(Box::new(count))
}

fn part2_viz(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    use gtk::{gdk, glib, prelude::*};

    struct State {
        id_ranges: &'static mut [ops::RangeInclusive<u64>],
        pending_merge: Option<[usize; 2]>,
        last_merge_target_i: Option<usize>,
    }

    impl State {
        const fn new() -> Self {
            Self {
                id_ranges: &mut [],
                pending_merge: None,
                last_merge_target_i: None,
            }
        }

        fn from_id_ranges(id_ranges: &'static mut [ops::RangeInclusive<u64>]) -> Self {
            Self {
                id_ranges,
                pending_merge: None,
                last_merge_target_i: None,
            }
        }
    }

    static STATE: sync::RwLock<State> = sync::RwLock::new(State::new());

    let mut id_ranges = Vec::new();

    for line in input.lines() {
        if line.is_empty() {
            break;
        }

        let (min, max) = line.split_once('-').unwrap();
        let start = min.parse::<u64>().unwrap();
        let end = max.parse().unwrap();
        id_ranges.push(start..=end);
    }

    // Sort by range start asc.
    id_ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()));

    id_ranges.drain(id_ranges.len() - TRIM..).count();

    // for r in &id_ranges {
    //     // for r in id_ranges.iter().flatten() {
    //     println!(
    //         "{}..={} ({})",
    //         *r.start(),
    //         *r.end(),
    //         *r.end() - *r.start() + 1
    //     );
    // }

    // const SCALE: f64 = 1. / ((1000_u64.pow(4) / 2) as f64);
    const SCALE: f64 = 1. / ((1000_u64.pow(4) / 8) as f64);
    // const BAR_HEIGHT: f64 = 4.;
    const BAR_HEIGHT: f64 = 8.;

    // const SCALE: f64 = 30.;
    // const BAR_HEIGHT: f64 = 8.;

    const MARGIN: f64 = 8.0;

    let max_range = id_ranges.iter().map(|r| *r.end()).max().unwrap() as f64 * SCALE;

    let width = max_range as i32 + 2 * MARGIN.ceil() as i32;

    let height = (id_ranges.len() as f64 * BAR_HEIGHT).ceil() as i32 + 2 * MARGIN.ceil() as i32;

    *STATE.write().unwrap() = State::from_id_ranges(id_ranges.leak());

    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| {
        let drawing_area = gtk::DrawingArea::builder()
            .content_width(width)
            .content_height(height)
            .build();

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("AoC - Year 2025 - Day 6 - Part 2")
            .default_width(width)
            .default_height(height)
            .resizable(false)
            .child(&drawing_area)
            .build();

        drawing_area.set_draw_func(move |_widget, ctx, w, h| {
            const RED: gdk::RGBA = gdk::RGBA::new(1., 0., 0., 1.);
            const GREEN: gdk::RGBA = gdk::RGBA::new(0., 0.8, 0., 1.);
            const BLUE: gdk::RGBA = gdk::RGBA::new(0., 0., 1., 1.);
            const LIGHT_BLUE: gdk::RGBA = gdk::RGBA::new(0., 0.7, 1., 1.);

            ctx.set_source_rgb(0., 0., 0.);
            ctx.rectangle(0., 0., w as f64, h as f64);
            ctx.fill().unwrap();

            ctx.set_source_rgb(1., 1., 1.);
            ctx.rectangle(
                MARGIN,
                MARGIN,
                w as f64 - 2. * MARGIN,
                h as f64 - 2. * MARGIN,
            );
            ctx.fill().unwrap();

            {
                let state = STATE.read().unwrap();
                let id_ranges = &*state.id_ranges;
                for (i, range) in id_ranges.iter().enumerate() {
                    let color = match state.pending_merge {
                        Some([source_i, _]) if source_i == i => &RED,
                        Some([_, target_i]) if target_i == i => &BLUE,
                        None if state.last_merge_target_i == Some(i) => &LIGHT_BLUE,
                        _ => &GREEN,
                    };
                    ctx.set_source_color(color);

                    let start = *range.start() as f64 * SCALE;
                    let end = *range.end() as f64 * SCALE;
                    let extent = end - start;
                    ctx.rectangle(
                        MARGIN + start,
                        MARGIN + i as f64 * BAR_HEIGHT,
                        extent,
                        BAR_HEIGHT - 2.,
                    );

                    ctx.fill().unwrap();
                }
            }
        });

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
                {
                    let mut state = STATE.write().unwrap();
                    let State {
                        id_ranges,
                        pending_merge,
                        last_merge_target_i,
                    } = &mut *state;

                    let mut found_overlap = false;
                    let mut max = 0;
                    let mut max_i = 0;

                    for i in 0..id_ranges.len() {
                        let range = &id_ranges[i];
                        let (range_start, range_end) = (*range.start(), *range.end());
                        if range_end == 0 {
                            continue;
                        }
                        if range_start <= max {
                            found_overlap = true;

                            let start = *id_ranges[max_i].start();
                            let end = cmp::max(max, range_end);

                            if pending_merge.is_none() {
                                *pending_merge = Some([i, max_i]);
                                break;
                            }

                            id_ranges[max_i] = start..=end;
                            id_ranges[i] = 0..=0;

                            *pending_merge = None;
                            *last_merge_target_i = Some(max_i);
                            break;
                        } else {
                            max = range_end;
                            max_i = i;
                        }
                    }

                    if !found_overlap {
                        println!("no overlaps found");
                    }
                }

                drawing_area.queue_draw();
            } else if keyval == gdk::Key::Return {
                println!();
                println!("Calculate sum:");
                let state = STATE.read().unwrap();
                let count = state
                    .id_ranges
                    .iter()
                    .filter(|r| *r.end() > 0)
                    .map(|r| *r.end() - *r.start() + 1)
                    .sum::<u64>();
                println!(":: {count}");

                drawing_area.queue_draw();
            }

            glib::Propagation::Proceed
        }

        window.present();
    });

    let exit_code = app.run_with_args(&[] as &[&str]);
    if exit_code != glib::ExitCode::SUCCESS {
        eprintln!("{exit_code:?}");
    }

    None
}
