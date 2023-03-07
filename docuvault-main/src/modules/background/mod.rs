pub mod conversion {
    use sea_orm::{entity::*, query::*, FromQueryResult};
    use comrak::ComrakOptions;
    use tokio::time::sleep;

    use crate::{AppState, entity};

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
