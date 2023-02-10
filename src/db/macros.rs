use bb8::Pool;
use bb8_redis::RedisConnectionManager;
#[derive(Debug)]
pub struct RedisSchemaHeader {
    pub scope: String,
    pub key: String,
    pub expire_at: Option<usize>,
    pub con: Pool<RedisConnectionManager>,
}
#[macro_export]
macro_rules! redis_schema {
    ( $header: expr, {$($( $col:ident:$type:ty )+ $(,)?)*} ) => {
        {
            use redis::AsyncCommands;
            use paste::paste;

            use crate::routes::error::GlobalError;

            #[allow(dead_code)]
            #[derive(Debug)]
            struct RedisSchema {
                header: RedisSchemaHeader,
                $($($col: Option<$type>)+ ),*,
            }
            paste!{
                impl RedisSchema {
                    #[allow(dead_code)]
                    async fn flush(&mut self) -> Result<&mut Self, GlobalError>{
                        let con = self.header.con.clone();
                        let mut con = con.get().await?;
                        $($(
                                let key = format!("{}:{}:{}",self.header.scope, self.header.key, stringify!($col));
                                if(self.$col.is_some()){
                                    let _: () = con.set(&key, self.$col.clone().unwrap()).await?;
                                }
                                if(self.header.expire_at.is_some()){
                                    let _: () = con.expire_at(&key, self.header.expire_at.clone().unwrap()).await?;
                                }
                           )+)*

                            Ok(self)
                    }
                    #[allow(dead_code)]
                    async fn del_all(&mut self) -> Result<&mut Self, GlobalError>{
                        let con = self.header.con.clone();
                        let mut con = con.get().await?;
                        $($(
                                let key = format!("{}:{}:{}",self.header.scope, self.header.key, stringify!($col));
                                let _: () = con.del(&key).await?;
                           )+)*
                            Ok(self)
                    }
                    $($(
                            #[allow(dead_code)]
                            fn [<set_ $col>](&mut self, target:$type) -> &mut Self{
                                self.$col = Some(target); 
                                self
                            }
                            #[allow(dead_code)]
                            async fn [<get_ $col>](&mut self) ->Result<&mut Self, GlobalError>{
                                let con = self.header.con.clone();
                                let mut con = con.get().await?;
                                let key = format!("{}:{}:{}",self.header.scope, self.header.key, stringify!($col));
                                let res: Option<$type> = con.get(key).await?;
                                self.$col = res;
                                Ok(self)
                            }
                       )+)*
                }

            }
            
            RedisSchema {
                header: $header,
                $($($col: None)+ ),*
            }
        }
    };
}

