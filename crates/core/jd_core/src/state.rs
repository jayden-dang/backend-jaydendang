use crate::ctx::CtxW;
use crate::Result;
use crate::{ctx::Ctx, ctx::CtxExtError, error::Error, ModelManager};

#[derive(Clone)]
pub struct AppState {
    pub ctx: CtxW,
    pub mm: ModelManager,
}

impl AppState {
    pub fn new(mm: ModelManager) -> Self {
        Self {
            ctx: CtxW(Ctx::root_ctx()),
            mm,
        }
    }

    // Tạo với user_id cụ thể
    pub fn new_with_user(mm: ModelManager, user_id: i64) -> Result<Self> {
        let ctx = if user_id == 0 {
            Ctx::root_ctx()
        } else {
            Ctx::new(user_id).map_err(|e| Error::CtxErr(CtxExtError::CtxCreateFail(e.to_string())))?
        };

        Ok(Self { ctx: CtxW(ctx), mm })
    }

    // Helper methods để truy cập
    pub fn ctx(&self) -> &Ctx {
        &self.ctx.0
    }

    pub fn ctx_w(&self) -> &CtxW {
        &self.ctx
    }
}
