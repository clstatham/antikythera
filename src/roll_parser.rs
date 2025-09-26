use crate::rules::dice::{Advantage, RollPlan, RollSettings};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, space0},
    combinator::{all_consuming, map, map_res, opt},
    sequence::{delimited, pair, preceded},
};

pub fn parse_roll(input: &str) -> anyhow::Result<RollPlan> {
    let res = all_consuming(roll_plan).parse(input);

    match res {
        Ok((_, roll_plan)) => Ok(roll_plan),
        Err(_) => Err(anyhow::anyhow!("Failed to parse roll plan")),
    }
}

fn roll_plan(input: &str) -> IResult<&str, RollPlan> {
    let (input, (num_dice, die_size, modifier, settings)) = (
        map_res(digit1, |s: &str| s.parse::<u32>()),
        preceded(char('d'), map_res(digit1, |s: &str| s.parse::<u32>())),
        opt(preceded(
            space0,
            pair(
                alt((char('+'), char('-'))),
                preceded(space0, map_res(digit1, |s: &str| s.parse::<i32>())),
            ),
        )),
        opt(preceded(space0, roll_settings)),
    )
        .parse(input)?;

    let modifier = match modifier {
        Some(('+', value)) => value,
        Some(('-', value)) => -value,
        None => 0,
        _ => unreachable!(),
    };

    let settings = settings.unwrap_or_else(RollSettings::default);

    Ok((
        input,
        RollPlan {
            num_dice,
            die_size,
            modifier,
            settings,
        },
    ))
}

fn roll_settings(input: &str) -> IResult<&str, RollSettings> {
    delimited(
        char('['),
        map(
            (
                opt(preceded(space0, advantage)),
                opt(preceded(space0, minimum_die_value)),
                opt(preceded(space0, maximum_die_value)),
                opt(preceded(space0, reroll_dice_below)),
            ),
            |(advantage, min, max, reroll)| RollSettings {
                advantage: advantage.unwrap_or(Advantage::Normal),
                minimum_die_value: min,
                maximum_die_value: max,
                reroll_dice_below: reroll,
            },
        ),
        preceded(space0, char(']')),
    )
    .parse(input)
}

fn advantage(input: &str) -> IResult<&str, Advantage> {
    alt((
        map(tag("adv"), |_| Advantage::Advantage),
        map(tag("dis"), |_| Advantage::Disadvantage),
    ))
    .parse(input)
}

fn minimum_die_value(input: &str) -> IResult<&str, u32> {
    preceded(tag("min="), map_res(digit1, |s: &str| s.parse::<u32>())).parse(input)
}
fn maximum_die_value(input: &str) -> IResult<&str, u32> {
    preceded(tag("max="), map_res(digit1, |s: &str| s.parse::<u32>())).parse(input)
}
fn reroll_dice_below(input: &str) -> IResult<&str, u32> {
    preceded(tag("rr<"), map_res(digit1, |s: &str| s.parse::<u32>())).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::dice::{Advantage, RollPlan, RollSettings};

    #[test]
    fn test_parse_roll_simple() {
        let input = "2d6+3";
        let expected = RollPlan {
            num_dice: 2,
            die_size: 6,
            modifier: 3,
            settings: RollSettings {
                advantage: Advantage::Normal,
                minimum_die_value: None,
                maximum_die_value: None,
                reroll_dice_below: None,
            },
        };
        let result = parse_roll(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_roll_with_settings() {
        let input = "4d10-2 [adv min=3 max=8 rr<2]";
        let expected = RollPlan {
            num_dice: 4,
            die_size: 10,
            modifier: -2,
            settings: RollSettings {
                advantage: Advantage::Advantage,
                minimum_die_value: Some(3),
                maximum_die_value: Some(8),
                reroll_dice_below: Some(2),
            },
        };
        let result = parse_roll(input).unwrap();
        assert_eq!(result, expected);
    }
}
