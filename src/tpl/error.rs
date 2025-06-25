use std::fmt::Display;

use winnow::error::{ContextError, ErrMode};

pub struct WinnowErrorEx(ErrMode<ContextError>);

impl Display for WinnowErrorEx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut context_vec: Vec<String> = match &self.0 {
            ErrMode::Incomplete(_) => {
                write!(f, "Incomplete input:",)?;
                Vec::new()
            }
            ErrMode::Backtrack(err) => {
                write!(f, "backtrack : ")?;
                if let Some(cause) = err.cause() {
                    write!(f, "cause: {}", cause)?;
                }
                collect_context(err)
            }
            ErrMode::Cut(err) => {
                write!(f, "cut: ")?;
                if let Some(cause) = err.cause() {
                    write!(f, "cause: {}", cause)?;
                }
                collect_context(err)
            }
        };
        context_vec.reverse();
        writeln!(f, "parse context:",)?;
        for context in context_vec {
            write!(f, "{}::", context)?;
        }
        Ok(())
    }
}

fn collect_context(err: &ContextError) -> Vec<String> {
    let mut context_vec = Vec::new();
    let current = err;

    for context in current.context() {
        match context {
            winnow::error::StrContext::Label(value) => {
                context_vec.push(value.to_string());
            }
            winnow::error::StrContext::Expected(value) => {
                context_vec.push(value.to_string());
            }
            _ => {}
        }
    }
    context_vec
}
impl From<ErrMode<ContextError>> for WinnowErrorEx {
    fn from(err: ErrMode<ContextError>) -> Self {
        WinnowErrorEx(err)
    }
}
pub fn err_code_prompt(code: &str) -> String {
    let take_len = if code.len() > 200 { 200 } else { code.len() };
    if let Some((left, _right)) = code.split_at_checked(take_len) {
        return format!("{}...", left);
    }
    "".to_string()
}
