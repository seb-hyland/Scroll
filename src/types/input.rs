#[derive(Clone, Debug)]
pub enum InputField {
    String { req: bool },
    Date { req: bool },
    One { id: String, req: bool },
    Multi { id: String, req: bool },
}



impl InputField {
    pub fn is_req(&self) -> bool {
        match self {
            InputField::String { req, .. } => *req,
            InputField::Date { req, .. } => *req,
            InputField::One { req, .. } => *req,
            InputField::Multi { req, .. } => *req,
        }
    }
}

