use std::{io, ops::Range};

use crop::Rope;

use syn::spanned::Spanned;
use thiserror::Error;

use crate::{
    formatter::collect::collect_macros_in_file,
    formatter::HtmlMacro,
    formatter::{format_macro, FormatterSettings},
};

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("could not read file")]
    IoError(#[from] io::Error),
    #[error("could not parse file")]
    ParseError(#[from] syn::Error),
}

#[derive(Debug)]
struct TextEdit {
    range: Range<usize>,
    new_text: String,
}

pub(crate) fn format_file_source(
    source: &str,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let ast = syn::parse_file(source)?;
    let macros = collect_macros_in_file(&ast);
    format_source(source, macros, settings)
}

fn format_source<'a>(
    source: &'a str,
    macros: Vec<HtmlMacro<'a>>,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let mut rope: Rope = source.parse().unwrap();
    let mut edits = Vec::new();

    for dom_mac in macros {
        let mac = dom_mac.inner();
        let start = mac.path.span().start();
        let end = mac.delimiter.span().close().end();
        let start_byte = line_column_to_byte(&rope, start);
        let end_byte = line_column_to_byte(&rope, end);
        let new_text = format_macro(&dom_mac, &settings, Some(source));

        edits.push(TextEdit {
            range: start_byte..end_byte,
            new_text,
        });
    }

    let mut last_offset: isize = 0;
    for edit in edits {
        let start = edit.range.start;
        let end = edit.range.end;
        let new_text = edit.new_text;

        rope.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    Ok(rope.to_string())
}

fn line_column_to_byte(source: &Rope, point: proc_macro2::LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn it_works() {
        let source = indoc! {r#"
            fn main() {
                html! {  <div>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                <div>
                    <span>"hello"</span>
                </div>
            }; 
        }

        "###);
    }

    #[test]
    fn for_each() {
        let source = indoc! {r#"
            fn main() {
                html! {  <div>
                    {for item in (0..3).enumerate() {
                        html! { <li>"Her name is Kitty White."</li> }
                    }}</div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                <div>
                    {for item in (0..3).enumerate() {
                        html! { <li>"Her name is Kitty White."</li> }
                    }}
                </div>
            }; 
        }

        "###);
    }

    #[test]
    fn with_comments() {
        let source = indoc! {r#"
            // comment outside dom macro
            fn main() {
                html! {  
                    // Top level comment
                    <div>  
                        // This is one beautiful message
                    <span>"hello"</span> // at the end of the line
                    <div>// at the end of the line
             // double
             // comments
                    <span>"hello"</span> </div>
                     <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
             key=|(index, _error)| *index // yeah
             />
             <div> // same line comment
             // with comment on the next line
             </div>
             // comments with empty lines inbetween

             // and some more
             // on the next line
                    </div>  }; 
            }

            // comment after dom macro
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        // comment outside dom macro
        fn main() {
            html! {
                // Top level comment
                <div>
                    // This is one beautiful message
                    // at the end of the line
                    <span>"hello"</span>
                    // at the end of the line
                    <div>
                        // double
                        // comments
                        <span>"hello"</span>
                    </div>
                    <For
                        // a function that returns the items we're iterating over; a signal is fine
                        each=move || { errors.clone().into_iter().enumerate() }
                        // a unique key for each item as a reference
                        // yeah
                        key=|(index, _error)| *index
                    />
                    // same line comment
                    <div>// with comment on the next line
                    </div>
                // comments with empty lines inbetween

                // and some more
                // on the next line
                </div>
            }; 
        }

        // comment after dom macro
        "###);
    }

    #[test]
    fn nested() {
        let source = indoc! {r#"
            fn main() {
                html! {  <div>  <span>{
                        let a = 12;

                        html! {             
                            
                                         <span>{a}</span>
                        }
                }</span></div>  };
            }            
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                <div>
                    <span>
                        {
                            let a = 12;
                            html! { <span>{a}</span> }
                        }
                    </span>
                </div>
            };
        }            
        "###);
    }

    #[test]
    fn nested_with_comments() {
        let source = indoc! {r#"
            fn main() {
                html! {  
                    // parent div
                    <div>  

                    // parent span
                    <span>{
                        let a = 12;

                        html! {             
                            // wow, a span
                            <span>{a}</span>
                        }
                }</span></div>  };
            }            
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                // parent div
                <div>

                    // parent span
                    <span>
                        {
                            let a = 12;
                            html! {
                                // wow, a span
                                <span>{a}</span>
                            }
                        }
                    </span>
                </div>
            };
        }            
        "###);
    }

    #[test]
    fn multiple() {
        let source = indoc! {r#"
            fn main() {
                html! {  <div>  <span>"hello"</span></div>  }; 
                html! {  <div>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                <div>
                    <span>"hello"</span>
                </div>
            }; 
            html! {
                <div>
                    <span>"hello"</span>
                </div>
            }; 
        }
        "###);
    }

    #[test]
    fn with_special_characters() {
        let source = indoc! {r#"
            fn main() {
                html! {  <div>  <span>"hello²💣"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            html! {
                <div>
                    <span>"hello²💣"</span>
                </div>
            }; 
        }
        "###);
    }

    #[test]
    fn inside_match_case() {
        let source = indoc! {r#"
            use hirola::prelude::*;

            enum ExampleEnum {
                ValueOneWithAReallyLongName,
                ValueTwoWithAReallyLongName,
            }

            #[component]
            fn Component(val: ExampleEnum) -> Dom {
                match val {
                    ExampleEnum::ValueOneWithAReallyLongName => 
                        html! {
                                                                    <div>
                                                                        <div>"Value One"</div>
                                                                    </div>
                                                                },
                    ExampleEnum::ValueTwoWithAReallyLongName =>  html! {
                                                                    <div>
                                                                        <div>"Value Two"</div>
                                                                    </div>
                                                                },
                };
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        use hirola::prelude::*;

        enum ExampleEnum {
            ValueOneWithAReallyLongName,
            ValueTwoWithAReallyLongName,
        }

        #[component]
        fn Component(val: ExampleEnum) -> Dom {
            match val {
                ExampleEnum::ValueOneWithAReallyLongName => 
                    html! {
                        <div>
                            <div>"Value One"</div>
                        </div>
                    },
                ExampleEnum::ValueTwoWithAReallyLongName =>  html! {
                        <div>
                            <div>"Value Two"</div>
                        </div>
                    },
            };
        }
        "###);
    }
}
