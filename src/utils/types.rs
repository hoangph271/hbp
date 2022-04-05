use mustache::{Data, MapBuilder};
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub struct HbpError {
    msg: String,
}
impl fmt::Display for HbpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}
impl Error for HbpError {
    fn description(&self) -> &str {
        &self.msg
    }
}
impl From<std::io::Error> for HbpError {
    fn from (std_error: std::io::Error) -> HbpError {
        error!("{}", std_error);
        HbpError::from_message("IO Error")
    }
}
impl From<regex::Error> for HbpError {
    fn from (regex_error: regex::Error) -> HbpError {
        error!("{}", regex_error);
        HbpError::from_message("Regex Error")
    }
}

impl HbpError {
    pub fn from_message(msg: &str) -> HbpError {
        HbpError {
            msg: String::from(msg),
        }
    }
    pub fn unimplemented () -> HbpError {
        HbpError {
            msg: String::from("unimplemented"),
        }
    }
}

pub type HbpResult<T> = Result<T, HbpError>;

// #region // * Markdown stuffs
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MarkdownMetadata {
    og_title: String,
    og_type: String,
    og_url: String,
    og_image: String,
}
impl MarkdownMetadata {
    pub fn of_markdown(markdown_path: &Path) -> Option<MarkdownMetadata> {
        let json_file_name = match markdown_path.file_name() {
            Some(file_name) => {
                let mut file_name = file_name.to_string_lossy().into_owned();
                file_name.push_str(".json");

                file_name
            }
            None => return None,
        };

        let mut json_path = markdown_path.to_owned();
        json_path.set_file_name(json_file_name);

        if json_path.exists() {
            if let Ok(mut file) = File::open(json_path) {
                let mut json = String::new();

                if file.read_to_string(&mut json).is_err() {
                    return None;
                }

                if let Ok(json) = serde_json::from_str::<MarkdownMetadata>(&json) {
                    return Some(json);
                }

                debug!(
                    "is_err: {:?}",
                    serde_json::from_str::<MarkdownMetadata>(&json)
                );
            }
        }

        None
    }

    pub fn to_mustache_data(&self) -> Data {
        MapBuilder::new()
            .insert_str("og_title", self.og_title.clone())
            .insert_str("og_type", self.og_type.clone())
            .insert_str("og_url", self.og_url.clone())
            .insert_str("og_image", self.og_image.clone())
            .build()
    }
}
// #endregion
