extern crate proc_macro;
extern crate quote;
extern crate syn;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;


#[derive(Default, Debug)]
struct _ParseEntry {
    ident: Option<TokenTree>,
    // id: Option<TokenTree>,
    cc: Option<TokenTree>,
    container: Option<TokenTree>,
    container_kind: Option<TokenTree>,
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
pub fn box_db(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let entries = parse_entries(input);

    let mut v = Vec::new();
    for e in entries {
        let ident = e.ident.unwrap();
        let id = byte_string_to_int_literal(e.cc.clone().unwrap());
        let cc = e.cc.unwrap();
        let cont = e.container.unwrap();
        let cont_kind = e.container_kind.unwrap();
        let full = e.full.unwrap();
        let descrip = e.descrip.unwrap();
        let path = e.path.unwrap();
        // println!("Ident: {:?}", ident);
        // println!("ID: {}", id);
        // println!("cc: {:?}", cc);
        // println!(" ");
        let q = quote!{ #ident, #id, #cc, #cont::#cont_kind, #full, #descrip, #path; };
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
        match pos {
            0 => {
                match tt { // Identifier
                    // TokenTree::Ident(i) =>  e.ident = Some(i),
                    TokenTree::Ident(_) =>  e.ident = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos += 1;
                        }
                    }
                    _ => (),
                }
            }
            1 => {  // CC
                match tt {
                    // TokenTree::Literal(l) => e.cc = Some(l),
                    TokenTree::Literal(_) => e.cc = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        }
                    }
                    _ => (),
                }
            }
            2 => { // Container (this is two identifiers Container::<ContainerType>)
                match tt {
                    TokenTree::Ident(_) =>  {
                        if e.container.is_none() {
                            // e.container = Some(i);
                            e.container = Some(tt);
                        } else {
                            // e.container_kind = Some(i);
                            e.container_kind = Some(tt);
                        }
                    }
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        }
                    }
                    _ => (),
                }
            }
            3 => { // Full
                match tt {
                    // TokenTree::Ident(i) =>  e.full = Some(i),
                    TokenTree::Ident(_) =>  e.full = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        }
                    }
                    _ => (),
                }
            }
            4 => { // Description
                match tt {
                    // TokenTree::Literal(l) => e.descrip = Some(l),
                    TokenTree::Literal(_) => e.descrip = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ',' {
                            pos +=1;
                        }
                    }
                    _ => (),
                }
            }
            5 => { // Path
                match tt {
                    // TokenTree::Literal(l) => e.path = Some(l),
                    TokenTree::Literal(_) => e.path = Some(tt),
                    TokenTree::Punct(p) => {
                        if p.as_char() == ';' {
                            pos = 0;
                            // println!("Entry: {:?}", e);
                            entries.push(e);
                            e = _ParseEntry {..Default::default()};
                        }
                    }
                    _ => (),
                }
            }
            _ => { // Should probably panic if we get here.
                pos = 0;
            }
        }
    }
    entries
}
