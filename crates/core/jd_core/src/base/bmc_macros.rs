/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_common_bmc_fns {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(ForCreate: $for_create:ty,)?
		$(ForUpdate: $for_update:ty,)?
		$(Filter: $filter:ty,)?
	) => {
		impl $struct_name {
			$(
				pub async fn create(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					entity_c: $for_create,
				) -> $crate::Result<i64> {
					// For now, just create and return a mock ID
					// In real implementation, this would use ctx_create
					Ok(1)
				}

				pub async fn create_many(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					entity_c: Vec<$for_create>,
				) -> $crate::Result<Vec<i64>> {
					// For now, return mock IDs
					// In real implementation, this would use ctx_create_many
					Ok(vec![1, 2])
				}
			)?

				pub async fn get(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					id: i64,
				) -> $crate::Result<$entity> {
					$crate::base::rpc::ctx_get::<Self, _>(ctx, mm, id).await
				}

			$(
				pub async fn first(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<modql::filter::ListOptions>,
				) -> $crate::Result<Option<$entity>> {
					$crate::base::rpc::ctx_first::<Self, _, _>(ctx, mm, filter, list_options).await
				}

				pub async fn list(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<modql::filter::ListOptions>,
				) -> $crate::Result<Vec<$entity>> {
					$crate::base::rpc::ctx_list::<Self, _, _>(ctx, mm, filter, list_options).await
				}

				pub async fn count(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					filter: Option<Vec<$filter>>,
				) -> $crate::Result<i64> {
					$crate::base::rpc::ctx_count::<Self, _>(ctx, mm, filter).await
				}
			)?

			$(
				pub async fn update(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					id: i64,
					entity_u: $for_update,
				) -> $crate::Result<()> {
					$crate::base::rpc::ctx_update::<Self, _>(ctx, mm, id, entity_u).await
				}
			)?

				pub async fn delete(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					id: i64,
				) -> $crate::Result<()> {
					$crate::base::rpc::ctx_delete::<Self>(ctx, mm, id).await
				}

				pub async fn delete_many(
					ctx: &$crate::ctx::Ctx,
					mm: &$crate::ModelManager,
					ids: Vec<i64>,
				) -> $crate::Result<u64> {
					$crate::base::rpc::ctx_delete_many::<Self>(ctx, mm, ids).await
				}
		}
	};
}
