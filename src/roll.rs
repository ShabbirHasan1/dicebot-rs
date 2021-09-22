use rand::Rng;
use regex::Regex;
use std::fmt::{Display, Formatter};
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::ApplicationCommand;

pub struct Roll {
    die: u16,
    count: u16,
    modifier: u16,
    gm: u16,
}

impl Roll {
    pub fn from_command(command: Box<ApplicationCommand>) -> Result<Roll, String> {
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
                        gm = value as u16
                    }
                }
                _ => {}
            }
        }
        Ok(Roll {
            die,
            count,
            modifier,
            gm,
        })
    }

    // Custom ID Format
    // 0000000     | 0000          | 0000             | 0
    // Die (7 bit) | Count (4 bit) | Modifier (4 bit) | GM (1 bit)

    pub fn to_custom_id(&self) -> String {
        let custom_id = ((self.die - 1 & 0x7F) << 9)
            + ((self.count - 1 & 0x0F) << 5)
            + ((self.modifier & 0x0F) << 1)
            + self.gm;
        custom_id.to_string()
    }

    pub fn from_custom_id(custom_id: String) -> Roll {
        let custom_id: u16 = custom_id.parse().unwrap();
        Roll {
            die: (custom_id >> 9 & 0x7F) + 1,
            count: (custom_id >> 5 & 0x0F) + 1,
            modifier: custom_id >> 1 & 0x0F,
            gm: custom_id & 1,
        }
    }

    pub fn ephemeral(&self) -> bool {
        self.gm != 0
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
