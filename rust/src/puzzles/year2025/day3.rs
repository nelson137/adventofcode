inventory::submit!(crate::days::DayModule::new("2025", 3).with_executors(
    crate::day_part_executors![part1],
    crate::day_part_executors![part2],
));

struct BatteryBank<'a>(&'a [u8]);

impl BatteryBank<'_> {
    fn max_joltage(&self) -> u32 {
        let mut tens_digit_i = 0;
        let mut tens_digit = self.0[tens_digit_i];

        // Find the max digit in bank excluding the last
        for i in tens_digit_i + 1..self.0.len() - 1 {
            if self.0[i] > tens_digit {
                tens_digit = self.0[i];
                tens_digit_i = i;
            }
        }

        // Find the next max digit in the bank after the tens digit
        let ones_digit = *self.0[tens_digit_i + 1..self.0.len()].iter().max().unwrap();

        // Convert to joltage value
        (tens_digit - b'0') as u32 * 10 + (ones_digit - b'0') as u32
    }

    fn max_joltage_high_power(&self) -> u64 {
        let mut max_joltage = 0;
        let mut max_digit_j = -1_i32;

        for i in (0..12).rev() {
            max_digit_j += 1;
            let mut max_digit = self.0[max_digit_j as usize];

            for j in max_digit_j as usize + 1..self.0.len() - i {
                if self.0[j] > max_digit {
                    max_digit = self.0[j];
                    max_digit_j = j as i32;
                }
            }

            max_joltage += (max_digit - b'0') as u64 * 10_u64.pow(i as u32);
        }

        max_joltage
    }
}

fn parse_battery_banks(input: &str) -> impl Iterator<Item = BatteryBank<'_>> {
    input.lines().map(str::as_bytes).map(BatteryBank)
}

fn part1(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut total_joltage = 0_u32;

    for bank in parse_battery_banks(input) {
        total_joltage += bank.max_joltage();
    }

    Some(Box::new(total_joltage))
}

fn part2(input: &str) -> Option<Box<dyn std::fmt::Display>> {
    let mut total_joltage = 0_u64;

    for bank in parse_battery_banks(input) {
        total_joltage += bank.max_joltage_high_power();
    }

    Some(Box::new(total_joltage))
}
