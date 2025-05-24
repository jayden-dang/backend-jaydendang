/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_common_bmc_fns {
    (
        DMC: $struct_name:ident,
        Entity: $entity:ty,
        $(ReqCreate: $req_create:ty,)?
        $(ResCreate: $res_create:ty,)?
        $(ReqUpdate: $req_update:ty,)?
        $(Filter: $filter:ty,)?
    ) => {
        use axum::{extract::{Path, Query, State}, Json};

        impl $struct_name {
            $(
                pub async fn create(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Json(entity_c): Json<$req_create>,
                ) -> Result<Json<$res_create>> {
                    Ok(Json(base::create::<Self, _, _>(ctx, mm, entity_c).await?))
                }

                pub async fn create_many(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Json(entity_c): Json<Vec<$req_create>>,
                ) -> Result<Json<Vec<$res_create>>> {
                    Ok(Json(base::create_many::<Self, _, _>(ctx, mm, entity_c).await?))
                }
            )?

                pub async fn get(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Path(id): Path<i64>,
                ) -> Result<Json<$entity>> {
                    Ok(Json(base::get::<Self, _>(ctx, mm, id).await?))
                }

            $(
                pub async fn first(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Query(filter): Query<Option<Vec<$filter>>>,
                    Query(list_options): Query<Option<ListOptions>>,
                ) -> Result<Json<Option<$entity>>> {
                    Ok(Json(base::first::<Self, _, _>(ctx, mm, filter, list_options).await?))
                }

                pub async fn list(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Query(filter): Query<Option<Vec<$filter>>>,
                    Query(list_options): Query<Option<ListOptions>>,
                ) -> Result<Json<Vec<$entity>>> {
                    Ok(Json(base::list::<Self, _, _>(ctx, mm, filter, list_options).await?))
                }

                pub async fn count(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Query(filter): Query<Option<Vec<$filter>>>,
                ) -> Result<Json<i64>> {
                    Ok(Json(base::count::<Self, _>(ctx, mm, filter).await?))
                }
            )?

            $(
                pub async fn update(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Path(id): Path<i64>,
                    Json(entity_u): Json<$req_update>,
                ) -> Result<()> {
                    base::update::<Self, _>(ctx, mm, id, entity_u).await
                }
            )?

                pub async fn delete(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Path(id): Path<i64>,
                ) -> Result<()> {
                    base::delete::<Self>(ctx, mm, id).await
                }

                pub async fn delete_many(
                    ctx: &Ctx,
                    mm: &ModelManager,
                    Path(ids): Path<Vec<i64>>,
                ) -> Result<u64> {
                    base::delete_many::<Self>(ctx, mm, ids).await
                }
        }
    };
}
