#[macro_export]
macro_rules! generate_rest_common_fn {
  (
    DMC: $struct_name:ident,
    Entity: $entity:ty,
    $(ReqCreate: $req_create:ty,)?
      $(ResCreate: $res_create:ty,)?
      $(ReqUpdate: $req_update:ty,)?
      $(Filter: $req_get_filter:ty,)?
  ) => {
    use serde_json::{json, Value};
    use modql::filter::ListOptions;

    impl $struct_name {
      $(
        pub async fn create(
          State(db): State<&ModelManager>,
          Json(req): Json<$req_create>
        ) -> Result<Json<$res_create>> {
          Ok(Json(rest::create::<Self, _, _>(db, req).await?))
        }

        pub async fn create_many(
          State(db): State<&ModelManager>,
          Json(req): Json<Vec<$req_create>>,
        ) -> Result<Json<Vec<$res_create>>> {
          Ok(Json(rest::create_many::<Self, _, _>(db, req).await?))
        }
      )?
        $(
          pub async fn get_by_id(
            State(db): State<&ModelManager>,
            Path(id): Path<i64>
          ) -> Result<Json<$entity>> {
            Ok(Json(rest::get_by_id::<Self, _>(&db, id).await?))
          }

          pub async fn get_by_sth(
            State(db): State<&ModelManager>,
            Query(query): Query<$req_get_filter>,
            Query(list_options): Query<ListOptions>,
          ) -> Result<Json<Option<$entity>>> {
            Ok(Json(rest::first::<Self, _, _>(&db, Some(query), Some(list_options)).await?))
          }

          pub async fn list(
            State(db): State<&ModelManager>,
            Query(query): Query<$req_get_filter>,
            Query(list_options): Query<ListOptions>,
          ) -> Result<Json<Value>> {
            let (users, pagination) = rest::list::<Self, _, $entity>(&db, Some(query), Some(list_options)).await?;

            Ok(Json(json!({
              "data": users,
              "metadata": pagination
            })))
          }

          pub async fn count(
            State(db): State<&ModelManager>,
            Query(query): Query<$req_get_filter>,
          ) -> Result<Json<i64>> {
            Ok(Json(rest::count::<Self, _>(&db, Some(query)).await?))
          }
        )?
        $(
          pub async fn update(
            State(db): State<&ModelManager>,
            Path(id): Path<i64>,
            Json(req): Json<$req_update>,
          ) -> Result<()> {
            rest::update::<Self, _>(&db, id, req).await
          }

          pub async fn delete(
            State(db): State<&ModelManager>,
            Path(req): Path<i64>
          ) -> Result<()> {
            rest::delete::<Self>(&db, req).await
          }

          pub async fn delete_many(
            State(db): State<&ModelManager>,
            Json(req): Json<Vec<i64>>
          ) -> Result<()> {
            rest::delete_many::<Self>(&db, req).await
          }
        )?
    }
  };
}
