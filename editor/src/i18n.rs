use std::fmt;
use std::fmt::{Debug, Formatter};
use fluent::{FluentBundle, FluentResource, FluentArgs};

use unic_langid::LanguageIdentifier;

#[derive(Debug)]
pub struct I18nError {
    message: String,
}

impl I18nError {
    fn new(message: String) -> I18nError {
        return I18nError { message };
    }
}

impl fmt::Display for I18nError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct Translator {
    bundle: FluentBundle<FluentResource>,
}

impl Translator {
    pub fn format(&self, id: &str, fluent_args: Option<&FluentArgs>) -> Result<String, I18nError> {
        let message = match self.bundle.get_message(id) {
            Some(message) => message,
            None => return Err(I18nError::new(String::from("Message not found"))),
        };
        let pattern = match message.value() {
            Some(pattern) => pattern,
            None => return Err(I18nError::new(String::from("Message has no pattern"))),
        };
        let mut errors = vec![];
        let translated_string = self.bundle.format_pattern(pattern, fluent_args, &mut errors).into_owned();
        if errors.len() > 0 {
            let mut error_string = String::new();
            for error in errors {
                error_string.push_str(&format!("{}\n", error));
            }
            return Err(I18nError::new(error_string));
        }
        return Ok(translated_string);
    }

    pub fn new(bundle: FluentBundle<FluentResource>) -> Translator {
        return Translator { bundle };
    }
}

pub fn init_i18n(language: LanguageIdentifier) -> Result<Translator, I18nError> {
    let ftl_string_opt = get_ftl_string(&language);
    return match ftl_string_opt {
        Some(ftl_string) => {
            let mut bundle = FluentBundle::new(vec![language]);
            let res = FluentResource::try_new(ftl_string);
            return match res {
                Ok(r) => {
                    return match bundle.add_resource(r) {
                        Ok(_) => Ok(Translator::new(bundle)),
                        Err(_) => Err(I18nError::new(String::from("Failed to add resource to bundle"))),
                    };
                }
                Err(err) => {
                    Err(I18nError::new(format!("Failed to parse FTL: {:?}", err.1[0])))
                }
            };
        }
        None => {
            Err(I18nError::new(String::from("No FTL file found for language")))
        }
    };
}

fn get_ftl_string(language: &LanguageIdentifier) -> Option<String> {
    let ftl_strings: Vec<(LanguageIdentifier, &str)> = vec![
        ("en-US".parse().unwrap(), include_str!("../cres/i18n/en_US.ftl"))
    ];
    for (lang, ftl_string) in ftl_strings.iter() {
        if lang == language {
            return Some(ftl_string.to_string());
        }
    }
    return None;
}