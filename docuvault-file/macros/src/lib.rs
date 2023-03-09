use proc_macro::{TokenStream};
use proc_macro2::{TokenStream as TokenStream2};
use quote::{quote, format_ident};
use syn::{DeriveInput, parse_macro_input, Data, parse::Parse, AttributeArgs, Lit, NestedMeta, MetaNameValue, Meta}; 
#[proc_macro_attribute]
pub fn redis_schema(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: AttributeArgs = parse_macro_input!(args);
    let scope = match args.into_iter().next() {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{ lit: Lit::Str(s), ..}))) => {
            s.value()
        },
        _ => panic!("need scope attribute.")
    };

    let input: DeriveInput = parse_macro_input!(input);
    let struct_name = &input.ident;
    let fields = match &input.data {
        Data::Struct(s) => s.fields.clone(),
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);
    let field_type = fields.iter().map(|field| &field.ty);
    let zipped = field_name.zip(field_type);

    let mut expanded_fields = TokenStream2::default();
    let mut default_fields = TokenStream2::default();
    zipped.clone().for_each(|(ident, ty)|{
        expanded_fields.extend::<TokenStream2>(quote!(pub #ident: Option<#ty>,));
        default_fields.extend::<TokenStream2>(quote!(#ident: None,));
    });


    let mut flush_method = quote!{};
    let mut del_all_method = quote!{};
    let mut get_all_method = quote!{};
    let mut set_methods = quote!{};
    let mut get_methods = quote!{};
    for (ident, ty) in zipped {
        if let Some(ident) = ident {
            flush_method = quote!{
                #flush_method
                let key = format!("{}:{}:{}",self.scope, self.header.key, stringify!(#ident));
                if(self.#ident.is_some()){
                    let _: () = con.set(&key, self.#ident.clone().unwrap()).await?;
                }
                if(self.header.expire_at.is_some()){
                    let _: () = con.expire_at(&key, self.header.expire_at.clone().unwrap()).await?;
                }
            };
            del_all_method = quote!{
                #del_all_method
                let key = format!("{}:{}:{}",self.scope, self.header.key, stringify!(#ident));
                let _: () = con.del(&key).await?;
            };
            get_all_method = quote!{
                #get_all_method
                let key = format!("{}:{}:{}",self.scope, self.header.key, stringify!(#ident));
                let res: Option<#ty> = con.get(key).await?;
                self.#ident = res;
            };
            let set_method = format_ident!("set_{}", ident);
            set_methods = quote!{
                #set_methods
                pub fn #set_method(&mut self, target: #ty) -> &mut Self{
                    self.#ident = Some(target);
                    self  
                }
            };
            let get_method = format_ident!("get_{}", ident);
            get_methods = quote!{
                #get_methods
                pub async fn #get_method(&mut self) -> Result<&mut Self, GlobalError> {
                    let con = self.header.con.clone();
                    let mut con = con.get().await?;
                    let key = format!("{}:{}:{}",self.scope, self.header.key, stringify!(#ident));
                    let res: Option<#ty> = con.get(key).await?;
                    self.#ident = res;
                    Ok(self)
                }
            }

        }
    }

    let output = quote!(
        #[derive(Debug)]
        pub struct #struct_name {
            scope: String,
            header: RedisSchemaHeader,
            #expanded_fields
        }

        impl #struct_name {
            pub fn new(header: RedisSchemaHeader) -> Self{
                Self {
                    scope: #scope.into(),
                    header,
                    #default_fields 
                }
            }
            pub async fn flush(&mut self) -> Result<&mut Self, GlobalError>{
                let con = self.header.con.clone();
                let mut con = con.get().await?;

                #flush_method
                Ok(self)
            }
            pub async fn del_all(&mut self) -> Result<&mut Self, GlobalError>{
                let con = self.header.con.clone();
                let mut con = con.get().await?;
                #del_all_method
                Ok(self)
            }
            pub async fn get_all(&mut self) -> Result<&mut Self, GlobalError>{
                let con = self.header.con.clone();
                let mut con = con.get().await?;
                
                #get_all_method
                Ok(self)
            }

            #set_methods
            #get_methods
        }
    );
    TokenStream::from(output)
}
