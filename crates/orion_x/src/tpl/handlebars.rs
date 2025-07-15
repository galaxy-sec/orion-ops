use handlebars::Handlebars;
use orion_error::ErrorOwe;
use serde::Serialize;

use super::TplResult;

pub struct TplHandleBars<'a> {
    handlebars: Handlebars<'a>,
}
impl TplHandleBars<'_> {
    pub fn init() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        Self { handlebars }
    }

    pub fn render_data<T: Serialize>(&self, template: &str, data: &T) -> TplResult<String> {
        let out_data = self.handlebars.render_template(template, data).owe_biz()?;
        Ok(out_data)
    }
}
