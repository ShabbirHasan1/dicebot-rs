use rand::Rng;
use regex::Regex;
use std::fmt::{Display, Formatter};
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::ApplicationCommand;

pub struct Roll {
    id: u64,
    die: u16,
    count: u16,
    modifier: u16,
    gm: u8,
}

impl Roll {
    pub fn from_command(command: Box<ApplicationCommand>) -> Result<Roll, String> {
        let id = if let Some(user) = command.user {
            user.id.0
        } else {
            command.member.unwrap().user.unwrap().id.0
        };
        let mut die = 0;
        let mut count = 1;
        let mut modifier = 0;
        let mut gm = 0;
        for option in command.data.options {
            match option {
                CommandDataOption::String { name, value } => {
                    if name == "dice" {
                        let re = Regex::new(r"^(\d*)[dD](\d+)$").unwrap();
                        if let Some(caps) = re.captures(value.as_str()) {
                            let c = caps.get(1).unwrap().as_str();
                            if !c.is_empty() {
                                let cc = c.parse().unwrap();
                                if cc < 1 {
                                    return Err("You can't roll less than one die!".to_string());
                                }
                                if cc > 8 {
                                    return Err("You can't roll more than eight dice!".to_string());
                                }
                                count = cc;
                            }
                            let dd = caps.get(2).unwrap().as_str().parse().unwrap();
                            if dd < 4 {
                                return Err(
                                    "Your dice can't have less than four faces!".to_string()
                                );
                            }
                            if dd > 120 {
                                return Err("Your dice can't have more than 120 faces!".to_string());
                            }
                            die = dd;
                        } else {
                            return Err(
                                "Please enter the dice you want to roll, e. g. `1d20` or `4d8`!"
                                    .to_string(),
                            );
                        }
                    }
                }
                CommandDataOption::Integer { name, value } => {
                    if name == "modifier" {
                        if value < 1 {
                            return Err("Your modifier can't be less than one!".to_string());
                        }
                        if value > 8 {
                            return Err("Your modifier can't be more than eight!".to_string());
                        }
                        modifier = value as u16
                    }
                }
                CommandDataOption::Boolean { name, value } => {
                    if name == "gm" {
                        gm = value as u8
                    }
                }
                _ => {}
            }
        }
        Ok(Roll {
            id,
            die,
            count,
            modifier,
            gm,
        })
    }

    // Custom ID Format
    // 0000000000000000000000000000000000000000000000000000000000000000 | 0000000     | 0000          | 0000             | 0
    // Discord ID (64 bit)                                              | Die (7 bit) | Count (4 bit) | Modifier (4 bit) | GM (1 bit)

    pub fn to_custom_id(&self) -> String {
        let mut custom_id = (self.id as u128) << 16;
        custom_id += (self.die as u128 - 1 & 127) << 9;
        custom_id += (self.count as u128 - 1 & 15) << 5;
        custom_id += (self.modifier as u128 & 15) << 1;
        custom_id += self.gm as u128;
        custom_id.to_string()
    }

    pub fn from_custom_id(custom_id: String) -> Roll {
        let custom_id: u128 = custom_id.parse().unwrap();
        Roll {
            id: (custom_id >> 16) as u64,
            die: ((custom_id >> 9 & 127) + 1) as u16,
            count: ((custom_id >> 5 & 15) + 1) as u16,
            modifier: (custom_id >> 1 & 15) as u16,
            gm: (custom_id & 1) as u8,
        }
    }

    pub fn ephemeral(&self) -> bool {
        self.gm != 0
    }

    pub fn is_from(&self, id: u64) -> bool {
        self.id == id
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rng = rand::thread_rng();
        if self.count == 1 {
            let result = rng.gen_range(1..self.die + 1);
            if self.modifier != 0 {
                write!(
                    f,
                    "Your result is **{} *+ {}* = {}**",
                    result,
                    self.modifier,
                    result + self.modifier
                )
            } else {
                write!(f, "Your result is **{}**", result)
            }
        } else {
            let mut results: Vec<String> = Vec::new();
            let mut result = 0;
            for _ in 0..self.count {
                let throw = rng.gen_range(1..self.die + 1);
                results.push(throw.to_string());
                result += throw;
            }
            if self.modifier != 0 {
                write!(
                    f,
                    "Your results are **({}) *+ {}* = {}**",
                    results.join(" + "),
                    self.modifier,
                    result + self.modifier
                )
            } else {
                write!(
                    f,
                    "Your results are **({}) = {}**",
                    results.join(" + "),
                    result
                )
            }
        }
    }
}
