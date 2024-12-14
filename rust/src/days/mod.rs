use anyhow::Result;

pub(crate) fn execute_day(day_i: u32) -> Result<()> {
    let input = crate::inputs::get_input(day_i)?;

    let day = DAY_EXECUTORS[day_i as usize];
    let answer1 = (day.0)(&input);
    let answer2 = (day.1)(&input);

    println!("1: {answer1}");
    println!("2: {answer2}");

    Ok(())
}

macro_rules! day_modules {
    ($( $day:ident ),+ $(,)?) => {
        $(
            mod $day;
        )+

        pub(crate) static CLI_DAY_VALUES: &[&str] = &[$(
            stringify!($day)
        ),+];

        fn __day0(_: &str) -> Box<dyn std::fmt::Display> {
            Box::new(0_u32)
        }

        static DAY_EXECUTORS: &[(
            for<'input> fn(&'input str) -> Box<dyn std::fmt::Display>,
            for<'input> fn(&'input str) -> Box<dyn std::fmt::Display>
        )] = &[
            (__day0, __day0),
            $( (self::$day::part1, self::$day::part2) ),+
        ];
    };
}

day_modules![day1, day2, day3];
