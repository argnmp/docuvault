pub mod conversion {
    use std::{cell::RefCell, io::{BufWriter, Bytes, Cursor}};

    use docx::{Docx, document::Paragraph};
    use sea_orm::{entity::*, query::*, FromQueryResult};
    use comrak::{ComrakOptions, nodes::{AstNode, Ast, NodeValue}, Arena, arena_tree::Node, parse_document};
    use tokio::time::sleep;
    use tonic::Request;

    use crate::{AppState, entity, modules::grpc::upload::{upload_client::UploadClient, UploadRequest, PreUploadRequest}};

    pub fn extension<'a>(c_type: i32)->&'a str{
        match c_type {
            0 => "html",
            1 => "html",
            2 => "txt",
            3 => "docx",
            4 => "pdf",
            5 => "epub",
            6 => "json",
            _ => "",
        }
    }


    pub fn convert_to_html(state: AppState, convert_id: (i32, i32), target: String){
        tokio::spawn(async move {
            //find ways for logging
            // let data = comrak::markdown_to_html(&payload.raw.clone(), &ComrakOptions::default());
            let data = comrak::markdown_to_html(&target, &ComrakOptions::default());
            let convert = entity::convert::Entity::find_by_id(convert_id)
                .one(&state.db_conn)
                .await
                .expect("convet db error");
            if convert.is_none() {
                panic!("convert instance not exists");
            }
            let mut convert: entity::convert::ActiveModel = convert.unwrap().into();
            convert.data = Set(Some(data));
            convert.status = Set(1);
            convert.update(&state.db_conn).await.expect("convert db fail");
        });
    }

    fn parse_raw<'a>(arena: &'a Arena<Node<'a, RefCell<Ast>>>, raw: String) -> &'a Node<'a, RefCell<Ast>> {
        let root = parse_document(
            arena,
            &raw[..],
            &ComrakOptions::default());
        root
    } 
    fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F)
        where F : FnMut(&'a AstNode<'a>) {
            f(node);
            for c in node.children() {
                iter_nodes(c, f);
            }
        }

    fn parse_to_txt(raw: String)->Vec<u8>{
        let arena = Arena::new();
        let root = parse_raw(&arena, raw);

        let mut txt = vec![];
        iter_nodes(root, &mut |node| {
            match &mut node.data.borrow_mut().value {
                &mut NodeValue::Text(ref mut text) => {
                    txt.append(&mut text.clone());
                }
                _ => (),
            }
        });
        txt
    }
    pub fn parse_to_docx(raw: String)->Vec<u8>{
        let mut docx = Docx::default();
        let arena = Arena::new();
        let root = parse_raw(&arena, raw);
        iter_nodes(root, &mut |node| {
            match &mut node.data.borrow_mut().value {
                &mut NodeValue::Text(ref mut text) => {
                    // create a new paragraph and insert it
                    let para = Paragraph::default().push_text(String::from_utf8(text.clone()).unwrap());
                    docx.document.push(para);
                }
                _ => (),
            }
        });
        let buf = Cursor::new(Vec::new());
        let writer = BufWriter::new(buf);
        let res = docx.write(writer).unwrap();
        let res = res.into_inner().unwrap().into_inner();
        res
    }
    pub async fn convert(state: AppState, convert_id: (i32, i32), c_type: i32){
        let file_proxy_addr = state.file_proxy_addr.lock().await.clone();
        tokio::spawn(async move {
            let docorg = entity::docorg::Entity::find_by_id(convert_id.0)
                .one(&state.db_conn)
                .await.expect("db connection failed");

            let docorg = match docorg { Some(docorg) => docorg,
                None => return (),
            };
            let (data, extension, ftype) = match c_type {
                2 => (parse_to_txt(docorg.raw), "txt", "text/plain"),
                3 => (parse_to_docx(docorg.raw), "docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                _ => panic!("not supported c_type"),
            };
            dbg!(&data);
            
            let mut upload_client = UploadClient::connect(file_proxy_addr).await.unwrap();
            let object_id = upload_client.pre_upload(Request::new(PreUploadRequest{
                name: format!("{}.{}", docorg.title, extension),
                ftype: ftype.to_owned(),
                size: data.len() as u64,
                docuser_id: docorg.docuser_id,
                data,
                object_id: None,
            })).await.unwrap();
            let object_id = object_id.into_inner().object_id;
            let _ = upload_client.upload(Request::new(UploadRequest { object_id: object_id.clone(), doc_id: convert_id.0 })).await;

            let convert = entity::convert::Entity::find_by_id(convert_id)
                .one(&state.db_conn)
                .await
                .expect("convet db error");
            let mut convert: entity::convert::ActiveModel = convert.unwrap().into();
            convert.data = Set(Some(object_id));
            convert.status = Set(1);
            convert.update(&state.db_conn).await.expect("convert db fail");
        });
    }
}

pub mod sanitize {
    use sea_orm::{entity::*, query::*, FromQueryResult};

    use crate::{AppState, entity, routes::error::GlobalError, modules::grpc::delete::{delete_client::DeleteClient, DeleteRequest}};
    pub fn sanitize(state: AppState, user_id: i32){
        tokio::spawn(async move {
            let res = entity::docfile::Entity::find()
                .filter(Condition::all()
                        .add(entity::docfile::Column::DocuserId.eq(user_id))
                        .add(entity::docfile::Column::IsFixed.eq(false))
                       )
                .column(entity::docfile::Column::ObjectId)
                .all(&state.db_conn)
                .await.unwrap();
            dbg!(&res);

            let mut delete_client = DeleteClient::connect("http://[::1]:8080").await.unwrap();
            delete_client.delete(tonic::Request::new(DeleteRequest {
                object_ids: res.iter().map(|o|o.object_id.clone()).collect::<Vec<_>>(),
            })).await;
            
            for obj in res {
                let obj = obj.into_active_model();
                obj.delete(&state.db_conn).await;
            }

            
        });
    }
}

