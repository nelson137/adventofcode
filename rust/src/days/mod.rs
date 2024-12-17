use anyhow::Result;

type DayPartExecutor = for<'input> fn(&'input str) -> Option<Box<dyn ::std::fmt::Display>>;

macro_rules! day_modules {
    ($( $day:ident ),+ $(,)?) => {
        $(
            mod $day;
        )+

        pub(crate) static CLI_DAY_VALUES: &[&str] = &[$(
            stringify!($day)
        ),+];

        static DAY_EXECUTORS: &[(DayPartExecutor, DayPartExecutor)] = &[$(
            (self::$day::part1, self::$day::part2)
        ),+];
    };
}

day_modules![day1, day2, day3, day4, day5];

pub(crate) fn execute_day(day_i: u32, input: String) -> Result<()> {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    if let Some(answer) = (executors.0)(&input) {
        println!("1: {answer}");
    }

    if let Some(answer) = (executors.1)(&input) {
        println!("2: {answer}");
    }

    Ok(())
}
