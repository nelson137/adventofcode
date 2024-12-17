use anyhow::Result;

type DayPartAnswer = Box<dyn ::std::fmt::Display>;
type DayPartExecutor = for<'input> fn(&'input str) -> Option<DayPartAnswer>;

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

pub(crate) fn execute_day(
    day_i: u32,
    input: String,
) -> Result<(Option<DayPartAnswer>, Option<DayPartAnswer>)> {
    let executors = DAY_EXECUTORS[(day_i - 1) as usize];

    let answer1 = (executors.0)(&input);
    let answer2 = (executors.1)(&input);

    Ok((answer1, answer2))
}
