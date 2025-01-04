use std::{fmt, iter, ops};

use itertools::Itertools;

crate::day_executors! {
    [part1]
    [part2]
}

crate::day_visualizers! {
    []
    []
}

fn parse(input: &str) -> Vec<Equation> {
    input
        .lines()
        .map(|line| {
            let Some((test_str, values_str)) = line.split_once(": ") else {
                panic!("Invalid input line: {line}");
            };
            let Ok(test) = test_str.parse::<u64>() else {
                panic!("Invalid test: {test_str}");
            };
            let Some(values) = values_str
                .split(" ")
                .map(|v| v.parse::<u64>().ok())
                .collect::<Option<Vec<u64>>>()
            else {
                panic!("Invalid values: {values_str}");
            };
            Equation { test, values }
        })
        .collect()
}

struct Equation {
    test: u64,
    values: Vec<u64>,
}

impl fmt::Display for Equation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:", self.test)?;
        for val in &self.values {
            write!(f, " {val}")?;
        }
        Ok(())
    }
}

impl Equation {
    fn try_solve(&self) -> Option<u64> {
        let nops = self.values.len() - 1;
        iter::repeat_n([Operator::Add, Operator::Mul], nops)
            .multi_cartesian_product()
            .any(|ops| self.is_solution(&ops))
            .then_some(self.test)
    }

    fn is_solution(&self, ops: &[Operator]) -> bool {
        let mut values = self.values.iter().copied();
        let v = values.next().unwrap();

        let maybe_solution = values
            .zip(ops.iter().copied())
            .fold(v, |acc, (v, op)| op.call(acc, v));

        maybe_solution == self.test
    }
}

#[derive(Clone, Copy)]
enum Operator {
    Add,
    Mul,
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Add => write!(f, "+"),
            Self::Mul => write!(f, "*"),
        }
    }
}

impl Operator {
    fn call<T: ops::Add<Output = T> + ops::Mul<Output = T>>(self, a: T, b: T) -> T {
        match self {
            Self::Add => ops::Add::add(a, b),
            Self::Mul => ops::Mul::mul(a, b),
        }
    }
}

pub(super) fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let calibration = parse(input);

    let answer = calibration
        .iter()
        .filter_map(Equation::try_solve)
        .sum::<u64>();

    Some(Box::new(answer))
}

pub(super) fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    _ = input;

    None
}