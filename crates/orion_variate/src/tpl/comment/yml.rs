use orion_error::{ErrorOwe, ErrorWith};
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::{fail, opt},
    error::{StrContext, StrContextValue},
    token::{literal, take_till, take_while},
    ModalResult, Parser,
};

pub struct YmlComment {}
impl YmlComment {
    pub fn remove(code: &str) -> TplResult<String> {
        remove_comment(code)
    }
}

use crate::tpl::{TplReason, TplResult};

use super::super::error::{err_code_prompt, WinnowErrorEx};
#[derive(Debug)]
pub enum YmlStatus {
    Comment,
    Code,
    StringDouble,
    StringSingle,
    BlockData,
}
pub fn ignore_comment_line(status: &mut YmlStatus, input: &mut &str) -> ModalResult<String> {
    let mut out = String::new();
    let mut line = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        match status {
            YmlStatus::Code => {
                let code = take_while(0.., |c| {
                    c != '"' && c != '|' && c != '\'' && c != '#' && c != '\n'
                })
                .parse_next(input)?;

                if opt("\n").parse_next(input)?.is_some() {
                    line += code;
                    if !line.trim().is_empty() {
                        out += line.as_str();
                        out += "\n";
                        line = String::new();
                    }
                    continue;
                }

                if opt("#").parse_next(input)?.is_some() {
                    if !code.trim().is_empty() {
                        line += code;
                    }
                    *status = YmlStatus::Comment;
                    continue;
                }
                line += code;
                if input.is_empty() {
                    break;
                }
                let rst = opt("|\n").parse_next(input)?;
                if let Some(tag_code) = rst {
                    line += tag_code;
                    *status = YmlStatus::BlockData;
                    continue;
                }
                let rst = opt("|").parse_next(input)?;
                if let Some(tag_code) = rst {
                    line += tag_code;
                    continue;
                }

                let rst = opt("\"").parse_next(input)?;
                if let Some(tag_code) = rst {
                    line += tag_code;
                    *status = YmlStatus::StringDouble;
                    continue;
                }
                let rst = opt("\'").parse_next(input)?;
                if let Some(tag_code) = rst {
                    line += tag_code;
                    *status = YmlStatus::StringSingle;
                    continue;
                }
                if opt("#").parse_next(input)?.is_some() {
                    *status = YmlStatus::Comment;
                    continue;
                }
                return fail.context(wn_desc("end-code")).parse_next(input);
            }
            YmlStatus::BlockData => match till_line_ending.parse_next(input) {
                Ok(data) => {
                    if data.trim().is_empty() {
                        *status = YmlStatus::Code;
                    } else {
                        line += data;
                    }
                }
                Err(e) => return Err(e),
            },

            YmlStatus::StringDouble => {
                let data = take_till(0.., |c| c == '"').parse_next(input)?;
                line += data;
                let data = literal("\"").parse_next(input)?;
                line += data;
                *status = YmlStatus::Code;
            }
            YmlStatus::StringSingle => {
                let data = take_till(0.., |c| c == '\'').parse_next(input)?;
                line += data;
                let data = literal("\'").parse_next(input)?;
                line += data;
                *status = YmlStatus::Code;
            }

            YmlStatus::Comment => {
                let _ = till_line_ending
                    .context(wn_desc("comment-line"))
                    .parse_next(input)?;
                //let _ = opt(line_ending) .context(wn_desc("comment-line_ending")) .parse_next(input)?;
                *status = YmlStatus::Code;
            }
        }
    }
    Ok(out)
}
#[inline(always)]
pub fn wn_desc(desc: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(desc))
}

pub fn remove_comment(code: &str) -> TplResult<String> {
    let mut xcode = code;
    let pure_code = ignore_comment(&mut xcode)
        .map_err(WinnowErrorEx::from)
        .owe(TplReason::Brief("yml comment error".into()))
        .position(err_code_prompt(code))
        .want("remove comment");
    match pure_code {
        Err(e) => {
            println!("code:\n{}", xcode);
            println!("{}", e);
            Err(e)
        }
        Ok(o) => Ok(o),
    }
}

pub fn ignore_comment(input: &mut &str) -> ModalResult<String> {
    let mut status = YmlStatus::Code;
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        //let mut line = till_line_ending.parse_next(input)?;
        let code = ignore_comment_line(&mut status, input)?;
        if !code.trim().is_empty() {
            out += code.as_str();
            if opt(line_ending).parse_next(input)?.is_some() {
                //out += "\n";
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {

    use std::{fs::read_to_string, path::PathBuf};

    use fs_extra::file::write_all;
    use orion_error::TestAssert;

    use super::remove_comment;

    #[test]
    fn test_case1() {
        let data = r#"
hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In
        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case2() {
        let data = r#"
            # Ranking of 1998 home runs
            ---
            - Mark McGwire
            - Sammy Sosa
            - Ken Griffey

            # Team ranking
            ---
            - Chicago Cubs
            - St Louis Cardinals
        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case4() {
        let data = r#"
    ---
    hr: # 1998 hr ranking
        - Mark McGwire
        - Sammy Sosa
    rbi:
        # 1998 rbi ranking
        - Sammy Sosa
        - Ken Griffey
        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case5() {
        let data = r#"
    ---
    unicode: "Sosa did fine.\u263A"
    control: "\b1998\t1999\t2000\n"
    hex esc: "\x0d\x0a is \r\n"

    single: '"Howdy!" he cried.'
    quoted: ' # Not a ''comment''.'
    tie-fighter: '|\-*-/|'
        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
        assert!(_codes.contains("#"));
    }

    #[test]
    fn test_case6() {
        let data = r#"
    ---
    application specific tag: !something |
     The #semantics of the tag
     above may be different for
     different documents.

    # hello
    galaxy is ok

        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
        assert!(_codes.contains("#"));
    }

    #[test]
    fn test_case7() {
        let data = r#"
global:
    imageRegistry: ""
    ## E.g.
    ## imagePullSecrets:
    ##   - myRegistryKeySecretName
    ##
    imagePullSecrets: []
    ## Security parameters
    ##
    security:
    ## @param global.security.allowInsecureImages Allows skipping image verification
    ##
    allowInsecureImages: false
            imageRegistry: ""
        "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
    }

    #[test]
    fn test_case8() {
        let data = r#"hello
# xxxabc"#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
    }
    //          regex: (\d+);((([0-9]+?)(\.|$)){4})
    #[test]
    fn test_case9() {
        //test this :
        //tag: !something |
        //regex

        let data = r#"regex: (\d+);((([0-9]+?)(\.|$)){4})"#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
    }
    #[test]
    fn test_case10() {
        let data = r#"
tunable:
# -- See the [property reference documentation](https://docs.redpanda.com/docs/reference/cluster-
log_segment_size_min: 16777216 # 16 mb
# -- See the property reference documentation.
log_segment_size_max: 268435456 # 256 mb
# -- See the property reference documentation.
compacted_log_segment_size: 67108864 # 64 mb
# -- See the property reference documentation.
max_compacted_log_segment_size: 536870912 # 512 mb
# -- See the property reference documentation.
kafka_connection_rate_limit: 1000
           "#;
        let _codes = remove_comment(data).assert();
        println!("{}", _codes);
    }
    #[test]
    fn test_file_case1() {
        let base_path = PathBuf::from("./test/data/yml");
        std::fs::create_dir_all(&base_path).assert();

        let val_file = base_path.join("values.yaml");
        let data = r#"
        tunable:
        # -- See the [property reference documentation](https://docs.redpanda.com/docs/reference/cluster-
        log_segment_size_min: 16777216 # 16 mb
        # -- See the property reference documentation.
        log_segment_size_max: 268435456 # 256 mb
        # -- See the property reference documentation.
        compacted_log_segment_size: 67108864 # 64 mb
        # -- See the property reference documentation.
        max_compacted_log_segment_size: 536870912 # 512 mb
        # -- See the property reference documentation.
        kafka_connection_rate_limit: 1000
                   "#;
        std::fs::write(&val_file, data).assert();

        let out_file = PathBuf::from("./test/data/yml/_values.yaml");
        let yml = read_to_string(&val_file).assert();
        let codes = remove_comment(yml.as_str()).assert();
        write_all(out_file, codes.as_str()).assert();
    }
}
