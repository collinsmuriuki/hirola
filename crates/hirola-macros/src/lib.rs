mod component;

use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprBlock, ExprForLoop, Ident, Stmt};
use syn_rsx::{Node, NodeType, ParserConfig};

fn to_token_stream(input: proc_macro::TokenStream) -> TokenStream {
    match syn_rsx::parse_with_config(input, ParserConfig::new()) {
        Ok(mut nodes) => {
            if nodes.len() == 1 {
                let node = nodes.pop().expect("unable to convert node to tokens");
                node_to_tokens(node)
            } else {
                fragment_to_tokens(nodes)
            }
        }
        Err(error) => error.to_compile_error(),
    }
}

fn fragment_to_tokens(nodes: Vec<Node>) -> TokenStream {
    let mut tokens = TokenStream::new();
    let children_tokens = children_to_tokens(nodes);
    tokens.extend(quote! {
            {
                let element: ::hirola::prelude::DomType = ::hirola::prelude::GenericNode::fragment();
                #children_tokens
                element
            }

    });
    tokens
}

fn node_to_tokens(node: Node) -> TokenStream {
    let mut tokens = TokenStream::new();

    // NodeType::Element nodes can't have no name
    let name = node.name_as_string();

    if let Some(name) = name {
        if name[0..1].to_lowercase() == name[0..1] {
            let attributes = node
                .attributes
                .iter()
                .map(attribute_to_tokens);

            let children_tokens = children_to_tokens(node.children);

            tokens.extend(quote! {
            
            {
                
                let element: ::hirola::prelude::DomType = ::hirola::prelude::GenericNode::element(#name);
                #children_tokens
                #(#attributes)*
                element
             }
        });
        } else {
            let fnname: Ident = syn::parse_str(&name).unwrap();

            let mut attributes = node
                .attributes
                .iter()
                .map(|attribute| match &attribute.value {
                    Some(expr) => {
                        let ident: proc_macro2::TokenStream =
                            attribute.name_as_string().unwrap().parse().unwrap();
                        quote! {
                            #ident: #expr
                        }
                    }
                    None => quote! {},
                })
                .collect::<Vec<TokenStream>>();
            if !node.children.is_empty() {
                let children_tokens = children_to_tokens(node.children);
                attributes.extend(vec![quote! {
                    children: { 
                        let element: ::hirola::prelude::DomType = ::hirola::prelude::GenericNode::fragment();
                        #children_tokens
                        ::hirola::prelude::TemplateResult::new(element)
                     }
                }]);
            }

            let quoted = if attributes.is_empty() {
                quote!({&#fnname })
            } else {
                quote!({ &#fnname {#(#attributes),*} })
            };
            tokens.extend(quote! {
            {
                #quoted.render().unwrap()
             }
        });
        }
    } else {
        tokens.extend(fragment_to_tokens(node.children));
    }
    tokens
}

fn attribute_to_tokens(attribute: &Node) -> TokenStream {
    match &attribute.value {
        Some(value) => {
            match attribute.node_type {
                NodeType::Block => {
                    quote! {
                        #value
                    }
                }
                NodeType::Attribute => {
                    // NodeType::Attribute nodes can't have no name
                    let name = attribute
                        .name_as_string()
                        .expect("attribute should have name");

                    if name.starts_with("on:") {
                        let name = name.replace("on:", "");
                        quote! {
                            ::hirola::prelude::GenericNode::event(
                                &element,
                                #name,
                                ::std::boxed::Box::new(#value),
                            );
                        }
                    } else if name.starts_with("mixin:") {
                        let name_space = name.replace("mixin:", "");
                        quote! {
                            let element = ::std::clone::Clone::clone(&element);
                            {
                                let element = ::std::clone::Clone::clone(&element);
                                #[allow(unused_braces)]
                                let res = hirola::prelude::Mixin::mixin(#value, #name_space, element);
                                if let Err(err) = res {
                                    let current_line = std::line!();
                                    let this_file = std::file!();
                                    web_sys::console::error_1(&format!("{}, LINE: {}, FILE: {}", err, current_line, this_file).into());
                                }
                            }

                        }
                    } else if &name == "ref" {
                        quote! {
                            ::hirola::prelude::NodeRef::set(
                                &#value,
                                ::std::clone::Clone::clone(&element),
                            );

                        }
                    } else {
                        let attribute_name = convert_name(&name);
                        quote! {
                            ::hirola::prelude::GenericNode::set_attribute(
                                &element,
                                #attribute_name,
                                &::std::format!("{}", #value),
                            );
                        }
                    }
                }
                _ => {
                    quote! {
                        compile_error!("Unexpected NodeType")
                    }
                }
            }
        }
        None => {
            let name = convert_name(
                &attribute
                    .name_as_string()
                    .expect("attribute should have a name"),
            );
            quote! {
                ::hirola::prelude::GenericNode::set_attribute(
                    &element,
                    #name,
                    &::std::format!(""),
                );
            }
        }
    }
}

fn children_to_tokens(children: Vec<Node>) -> TokenStream {
    let mut append_children = Vec::new();
    let mut tokens = TokenStream::new();
    if !children.is_empty() {
        for child in children {
            match child.node_type {
                NodeType::Element => {
                    let node = node_to_tokens(child);
                    append_children.extend(quote! {
                        ::hirola::prelude::GenericNode::append_child(&element, &#node);
                    });
                }
                NodeType::Text => {
                    let s = child
                        .value_as_string()
                        .expect("expecting a string on a text node");
                    append_children.extend(quote! {
                        ::hirola::prelude::GenericNode::append_child(
                            &element,
                            #[allow(unused_braces)]
                            &::hirola::prelude::GenericNode::text_node(#s),
                        );
                    });
                }
                NodeType::Comment => {
                    // let s = child
                    //     .value_as_string()
                    //     .expect("expecting a string on a comment node");
                    // tokens.extend(quote! {
                    //     #receiver.push(sauron_core::prelude::html::comment(#s));
                    // });
                }
                NodeType::Doctype => {
                    // let value = child
                    //     .value_as_string()
                    //     .expect("expecting a string value on a doctype");
                    // tokens.extend(quote! {
                    //     #receiver.push(sauron_core::prelude::html::doctype(#value));
                    // });
                }
                NodeType::Block => match child.value {
                    Some(syn::Expr::Block(expr)) => match braced_for_loop(&expr) {
                        Some(ExprForLoop {
                            pat, expr, body, ..
                        }) => {
                            append_children.extend(quote! {
                                for #pat in #expr {
                                    ::hirola::prelude::GenericNode::append_child(
                                        &element,
                                        &#body.inner_element(),
                                    );
                                }
                            });
                        }
                        _ => {
                            append_children.extend(quote! {
                                ::hirola::prelude::GenericNode::append_render(
                                    &element,                                    
                                    ::std::boxed::Box::new(move || {
                                        #[allow(unused_braces)]
                                        ::std::boxed::Box::new(#expr)
                                    }),
                                );
                            });
                        }
                    },
                    _ => {
                        return quote! {
                            compile_error!("Unexpected missing block for NodeType::Block")
                        }
                    }
                },
                _ => {
                    return quote! {
                        compile_error!(format!("Unexpected NodeType for child: {}", child.node_type))
                    }
                }
            }
        }
    } else {
        // tokens.extend(quote! {
        //     let #receiver = Vec::new();
        // });
    }

    let quoted = quote! {
        // let element = #tag_name;
        // #(#set_attributes)*
        // #(#set_event_listeners)*
        // #(#set_noderefs)*
        #(#append_children)*
        // element
    };
    tokens.extend(quoted);
    tokens
}

fn braced_for_loop(expr: &ExprBlock) -> Option<&ExprForLoop> {
    let len = expr.block.stmts.len();
    if len != 1 {
        None
    } else {
        let stmt = &expr.block.stmts[0];
        match stmt {
            Stmt::Expr(Expr::ForLoop(expr)) => Some(expr),
            _ => None,
        }
    }
}

fn convert_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());

    for c in name.trim_matches('_').chars() {
        match c {
            '_' => out.push('-'),
            c => out.push(c),
        }
    }

    out
}

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = to_token_stream(input);

    let quoted = quote! {
        ::std::convert::Into::<_>::into(#output)
    };
    quoted.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let f = parse_macro_input!(item as syn::ItemFn);
    component::create_function_component(f)
}
