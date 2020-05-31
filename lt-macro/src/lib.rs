extern crate proc_macro;
extern crate quote;
extern crate syn;
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;


#[derive(Default, Debug, Clone)]
struct _ParseEntry {
    ident: Option<TokenTree>,
    // id: Option<TokenTree>,
    cc: Option<TokenTree>,
    container: Option<TokenTree>,
    container_kind: Option<TokenTree>,
    special_value: Option<Group>,
    full: Option<TokenTree>,
    descrip: Option<TokenTree>,
    path: Option<TokenTree>,
}

// The intensity of the complication here .....
fn byte_string_to_int_literal(tt: TokenTree)  -> syn::LitInt {
    let lbs: syn::LitByteStr = syn::parse2(TokenStream::from(tt)).unwrap();
    let mut cc: [u8;4] = [0;4];
    cc.copy_from_slice(lbs.value().as_slice());
    let val = format!("0x{:0x?}", u32::from_be_bytes(cc)); // tob be extra obnoxious consider breaking this into "0xAA_BB_CC_DD"
    // println!("val: {}", &val);
    let li = syn::LitInt::new(&val,Span::call_site()); 
    // println!("LitInt: {:?}", li.to_string());
    li    // match &tt {    //     TokenTree::Literal(l) => {    //         println!("Struct: {:?}", l);    //         println!("Display: {}", l);    //         // let is = format!("Int {}", u32::from_be_bytes(l.to_string().as_bytes()));    //         // println!("IntString {}", is);    //     }    //     _ => println!("Not Literal: {:?}", tt),    // }    // tt
}

#[proc_macro]
pub fn define_boxes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let entries = parse_entries(input);

    let mut v = Vec::new();
    for e in entries {
        let ident = e.ident.expect("Failed an ident");
        let id = byte_string_to_int_literal(e.cc.clone().expect("Failed on character code"));
        let cc = e.cc.expect("Failed on character code");
        let cont = e.container.expect("Failed on container type");
        let cont_kind = e.container_kind.expect("Failed on Container Kind");
        let full = e.full.expect("failed on full boolean");
        let descrip = e.descrip.expect("failed on description");
        let path = e.path.expect("failed on path");
        let q;
        // println!("Quoting ident: {}", ident);
        if e.special_value.is_none() {
            q = quote!{ #ident, #id, #cc, #cont::#cont_kind, #full, #descrip, #path; };
        } else {
            let val = e.special_value.unwrap();
            q = quote!{ #ident, #id, #cc, #cont::#cont_kind(#val), #full, #descrip, #path; };
        }
        v.push(q);
    }

    let output = quote! {
        def_boxes! {
            #(#v)*
        }
    };

    // proc_macro::TokenStream::from(output)
    output.into()
}

fn parse_entries(input: TokenStream) -> Vec<_ParseEntry> {
    let mut entries = Vec::new();
    let mut e = _ParseEntry {..Default::default()};
    let mut pos = 0;
    // Brute force parser based on columns position.
    // I'm sure there's a more elegant way to do this.
    for tt in input {
        // println!("tt: {:?}\nPos: {}, e_val {:?}\n", &tt, pos, e_vals);
        // println!("tt: {:?}\nPos: {}, e_val {:?}\n", &tt, pos, e);
        // println!("tt: {:?}", tt);
        match pos {
            0 => {
                match tt { // Identifier
                    // TokenTree::Ident(i) =>  e.ident = Some(i),
                    TokenTree::Ident(_) => {
                        // println!("Identifier: {}", &tt);
                        e.ident = Some(tt);
                    }
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos += 1;
                        } else {
                            panic!(format!("Syntax error - non-ident, or missing comma {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error - non-ident, or missing comma at {}", &tt)),
                }
            }
            1 => {  // CC
                match tt {
                    // TokenTree::Literal(l) => e.cc = Some(l),
                    TokenTree::Literal(_) => e.cc = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        } else {
                            panic!(format!("Syntax error - non-ident, or missing comma {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error - non-ident, or missing comma at {}", &tt)),
                }
            }
            2 => { 
                // Container (this is two identifiers Container::<ContainerType>)
                // And in the case of Continertype == Special(u32), it has the integer
                // to retrieve.
                match tt {
                    TokenTree::Ident(_) =>  {
                        // println!("Container Ident: {}", tt);
                        if e.container.is_none() {
                            // e.container = Some(i);
                            e.container = Some(tt);
                        } else {
                            // println!("Container Type Ident: {}", tt);
                            // e.container_kind = Some(i);
                            e.container_kind = Some(tt);
                        }
                    }
                    TokenTree::Group(g) =>{
                        // println!("Container Group: {}", &tt);
                        // println!("Container Group: {}", &g);
                        // println!("Container Group Stream: {}", &g.stream());
                        e.special_value = Some(g);
                    }
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        } else if p.as_char() != ':' {
                            panic!(format!("Missing comma: {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error - non-ident, or missing comma at {}", &tt)),
                }
            }
            3 => { // Full
                match tt {
                    // TokenTree::Ident(i) =>  e.full = Some(i),
                    TokenTree::Ident(_) =>  e.full = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        } else {
                            panic!(format!("Missing comma: {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error - non-ident, or missing comma at {}", &tt)),
                }
            }
            4 => { // Description
                match tt {
                    // TokenTree::Literal(l) => e.descrip = Some(l),
                    TokenTree::Literal(_) => e.descrip = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        } else {
                            panic!(format!("Missing comma: {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error non-literal or missing comma: at {}", &tt)),
                }
            }
            5 => { // Path
                match tt {
                    // TokenTree::Literal(l) => e.path = Some(l),
                    TokenTree::Literal(_) => e.path = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ';' {
                            pos = 0;
                            // println!("Entry: {:?}\n", e);
                            entries.push(e);
                            e = _ParseEntry {..Default::default()};
                        } else {
                            panic!(format!("Missing semi-colon: {:?} at {:?}", p, &e.ident));
                        }
                    }
                    _ => panic!(format!("Syntax error non-literal or missing semi-colon: at {:?}", &e.ident)),
                }
            }
            _ => {
                panic!("Syntax error - too  many columns? Shouldn't have gotten here. {}", &tt);
            }
        }
    }
    entries
}
