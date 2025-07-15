use derive_getters::Getters;
use orion_error::{ErrorOwe, ErrorWith};
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::{fail, opt},
    error::{StrContext, StrContextValue},
    token::{literal, take_till, take_until, take_while},
    ModalResult, Parser,
};

use crate::tpl::{TplReason, TplResult};

use super::super::error::{err_code_prompt, WinnowErrorEx};

#[derive(Debug, Clone, Getters)]
pub struct CommentLabel {
    line: &'static str,
    beg: &'static str,
    end: &'static str,
}
impl CommentLabel {
    pub fn c_style() -> Self {
        Self {
            line: "//",
            beg: "/*",
            end: "*/",
        }
    }
}
pub struct CStyleComment {}
impl CStyleComment {
    pub fn remove(code: &str) -> TplResult<String> {
        remove_comment(code, &CommentLabel::c_style())
    }
}

#[derive(Debug)]
pub enum CppStatus {
    Comment,
    MultiComment,
    Code,
    StringData,
    RawString,
}
pub fn ignore_comment_line(
    status: &mut CppStatus,
    input: &mut &str,
    label: &CommentLabel,
) -> ModalResult<String> {
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        match status {
            CppStatus::Code => {
                let code =
                    take_while(0.., |c| c != '"' && c != '/' && c != '`').parse_next(input)?;

                if opt(label.line).parse_next(input)?.is_some() {
                    if !code.trim().is_empty() {
                        out += code;
                    }
                    *status = CppStatus::Comment;
                    continue;
                }
                if opt(label.beg).parse_next(input)?.is_some() {
                    if !code.trim().is_empty() {
                        out += code;
                    }
                    *status = CppStatus::MultiComment;
                    continue;
                }
                out += code;
                if input.is_empty() {
                    break;
                }
                let rst = opt("^\"").parse_next(input)?;
                if let Some(code) = rst {
                    out += code;
                    *status = CppStatus::RawString;
                    continue;
                }

                let rst = opt("\"").parse_next(input)?;
                if let Some(code) = rst {
                    out += code;
                    *status = CppStatus::StringData;
                    continue;
                }
                return fail.context(wn_desc("end-code")).parse_next(input);
            }
            CppStatus::RawString => match opt(take_until(0.., "\"^")).parse_next(input)? {
                Some(data) => {
                    out += data;
                    let data = "\"^".parse_next(input)?;
                    out += data;
                    *status = CppStatus::Code;
                }
                None => {
                    let data = till_line_ending.parse_next(input)?;
                    out += data;
                }
            },

            CppStatus::StringData => {
                let data = take_till(0.., |c| c == '"').parse_next(input)?;
                out += data;
                let data = "\"".parse_next(input)?;
                out += data;
                *status = CppStatus::Code;
            }
            CppStatus::Comment => {
                //TODO: 或到字符串结束
                let _ = till_line_ending.parse_next(input)?;
                *status = CppStatus::Code;
            }
            CppStatus::MultiComment => match opt(take_until(0.., label.end)).parse_next(input)? {
                Some(_) => {
                    let _ = literal(label.end).parse_next(input)?;
                    *status = CppStatus::Code;
                }
                None => {
                    let _ = till_line_ending.parse_next(input)?;
                }
            },
        }
    }
    Ok(out)
}
#[inline(always)]
pub fn wn_desc(desc: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(desc))
}

pub fn remove_comment(code: &str, comment: &CommentLabel) -> TplResult<String> {
    let mut xcode = code;
    let pure_code = ignore_comment(&mut xcode, comment)
        .map_err(WinnowErrorEx::from)
        .owe(TplReason::Brief("c style comment error".into()))
        .position(err_code_prompt(code))
        .want("remove comment");
    match pure_code {
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
        Ok(o) => Ok(o),
    }
}

pub fn ignore_comment(input: &mut &str, label: &CommentLabel) -> ModalResult<String> {
    let mut status = CppStatus::Code;
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        let code = ignore_comment_line(&mut status, input, label)?;
        out += code.as_str();
        if opt(line_ending).parse_next(input)?.is_some() {
            match status {
                CppStatus::MultiComment => {}
                CppStatus::RawString => {}
                _ => {
                    out += "\n";
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use super::*;
    #[test]
    fn test_comment() {
        let mut data = "hello //xxx\nboy";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "hello \nboy");

        let mut data = "	// need galaxy 0.4.1";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "");

        let mut data = "\"hello //\"\nboy";
        let expect = data;
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, expect);

        let mut data = "\"hello //\"//xxx\nboy";
        let expect = "\"hello //\"\nboy";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, expect);
    }

    #[test]
    fn test_multi_line_comment() {
        let mut data = "hello /* multi-line \n comment */ world";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "hello  world");

        let mut data = "hello /* multi-line \n comment */ world\n// single-line comment";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "hello  world\n");

        let mut data =
            "hello /* multi-line \n comment */ world\n/* another multi-line \n comment */";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "hello  world\n");
    }

    #[test]
    fn test_comment_in_string() {
        let mut data = "\"hello /* not a comment */ world\"";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "\"hello /* not a comment */ world\"");

        let mut data = "\"hello // not a comment\"\nworld";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "\"hello // not a comment\"\nworld");

        let mut data = "\"hello /* not a comment */ world\"\n// single-line comment";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "\"hello /* not a comment */ world\"");
    }

    #[test]
    fn test_mixed_comments_and_code() {
        let mut data = "code /* comment */ more code // another comment\nfinal code";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "code  more code \nfinal code");

        let mut data = "code /* comment */ more code /* another comment */ final code";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "code  more code  final code");
    }

    #[test]
    fn test_empty_string() {
        let mut data = "";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "");
    }

    #[test]
    fn test_only_comments() {
        let mut data = "// single-line comment";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "");

        let mut data = "/* multi-line comment */";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "");

        let mut data = "// single-line comment\n/* multi-line comment */";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "");
    }

    #[test]
    fn test_only_code() {
        let mut data = "code";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "code");

        let mut data = "code\nmore code";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "code\nmore code");
    }

    #[test]
    fn test_only_data() {
        let mut data = "\"data\"";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "\"data\"");

        let mut data = "\"data\"\n\"more data\"";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "\"data\"\n\"more data\"");
    }

    #[test]
    fn test_only_raw_data() {
        let mut data = "^\"raw data\"^";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "^\"raw data\"^");

        let mut data = "^\"raw data\"^\n^\"more raw data\"^";
        let codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
        assert_eq!(codes, "^\"raw data\"^\n^\"more raw data\"^");
    }

    #[test]
    fn test_complex_mixed_case1() {
        let mut data = r#"
        code /* multi-line comment */ "string with // comment"
        // single-line comment
        "raw data with /* comment */"
        /* another multi-line comment */
        more code
        "#;
        let _codes = ignore_comment(&mut data, &CommentLabel::c_style()).assert();
    }
    #[test]
    fn test_complex_mixed_case2() {
        let data = r#" code /* multi-line comment */ "string with // comment"
        // single-line comment
        /* another multi-line comment */
        more code
        "#;
        let _codes = remove_comment(data, &CommentLabel::c_style()).assert();
        println!("{}", _codes);
    }
}
