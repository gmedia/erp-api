use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize, Clone, Debug)]
pub struct StoreTask {
    #[validate(
        required(message = "Title content is a mandatory field."),
        length(min = 5, message = "Task title must be at least 5 characters long.")
    )]
    pub title: Option<String>,

    #[validate(
        required(message = "Task description is a mandatory field."),
        length(
            min = 10,
            message = "Task description must be at least 10 characters long."
        )
    )]
    pub description: Option<String>,
}
