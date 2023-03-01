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
