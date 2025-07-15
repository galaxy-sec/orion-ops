use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith};
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::opt,
    ModalResult, Parser,
};

use super::{
    comment::CommentFmt,
    error::{err_code_prompt, WinnowErrorEx},
    TplReason, TplResult,
};

const PROTECTED_BEG: &str = "!<!";
const PROTECTED_END: &str = "!>!";
#[derive(Debug, Clone, Getters)]
pub struct LabelCoverter {
    orion_label_beg: String,
    orion_label_end: String,
    target_label_beg: String,
    target_label_end: String,
}

impl LabelCoverter {
    pub fn new<S: Into<String>>(orion: (S, S), target: (S, S)) -> Self {
        Self {
            orion_label_beg: orion.0.into(),
            orion_label_end: orion.1.into(),
            target_label_beg: target.0.into(),
            target_label_end: target.1.into(),
        }
    }
    fn remvoe_comment(&self, cfmt: &CommentFmt, code: &str) -> TplResult<String> {
        let xcode = code;
        let pure_code = cfmt.remove(xcode).want("remove comment")?;
        Ok(pure_code)
    }

    //code 为多行的数据， 注释不进行转换, 注释的类型有行和块两种
    pub fn convert(&self, cfmt: &CommentFmt, code: String) -> TplResult<String> {
        let pure_code = self.remvoe_comment(cfmt, code.as_str())?;
        let coverted = convert_label(
            &mut pure_code.as_str(),
            vec![
                (self.target_label_beg(), PROTECTED_BEG),
                (self.target_label_end(), PROTECTED_END),
                (self.orion_label_beg(), self.target_label_beg()),
                (self.orion_label_end(), self.target_label_end()),
            ],
        )
        .map_err(WinnowErrorEx::from)
        .owe(TplReason::Brief("covert".into()))
        .position(err_code_prompt(pure_code.as_str()))
        .want("covert tpl label")?;
        Ok(coverted)
    }
    pub fn restore(&self, code: String) -> TplResult<String> {
        let coverted = convert_label(
            &mut code.as_str(),
            vec![
                (self.target_label_beg(), self.orion_label_beg()),
                (self.target_label_end(), self.orion_label_end()),
                (PROTECTED_BEG, self.target_label_beg()),
                (PROTECTED_END, self.target_label_end()),
            ],
        )
        .map_err(WinnowErrorEx::from)
        .owe(TplReason::Brief("restore!".into()))
        .position(err_code_prompt(code.as_str()))
        .want("covert tpl label")?;
        Ok(coverted)
    }
}

pub fn convert_label(input: &mut &str, dat: Vec<(&str, &str)>) -> ModalResult<String> {
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        let mut line = till_line_ending.parse_next(input)?;
        let mut for_line;
        for (f, t) in &dat {
            for_line = line.replace(f, t);
            line = for_line.as_str();
        }
        out += line;
        if opt(line_ending).parse_next(input)?.is_some() {
            out += "\n";
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;

    use super::*;

    #[test]
    fn test_convert_code_only() {
        let converter = LabelCoverter::new(("{{", "}}"), ("[[", "]]"));
        let input = "start {{var}} end [[hello]]";
        let output = converter
            .convert(&CommentFmt::CStyle, input.into())
            .assert();
        assert_eq!(output, "start [[var]] end !<!hello!>!");
        let output = converter.restore(output).assert();
        assert_eq!(output, "start {{var}} end [[hello]]");
    }

    #[test]
    fn test_convert_comment_only() {
        let converter = LabelCoverter::new(("{{", "}}"), ("[[", "]]"));
        let input = "start /* comment {{var}} */ end";
        let output = converter
            .convert(&CommentFmt::CStyle, input.into())
            .assert();
        assert_eq!(output, "start  end");
    }

    #[test]
    fn test_convert_mixed() {
        let converter = LabelCoverter::new(("{{", "}}"), ("[[", "]]"));
        let input = "code {{var}} /* comment {{var}} */ code {{var}}";
        let output = converter
            .convert(&CommentFmt::CStyle, input.into())
            .assert();
        assert_eq!(output, "code [[var]]  code [[var]]");
    }

    #[test]
    fn test_convert_hash_comment() {
        let converter = LabelCoverter::new(("{{", "}}"), ("[[", "]]"));
        let input = "# This is a comment\n{{ var }} # Another comment";
        let expected = "# This is a comment\n[[ var ]] # Another comment";
        assert_eq!(
            converter
                .convert(&CommentFmt::CStyle, input.into())
                .assert(),
            expected
        );
    }
}
