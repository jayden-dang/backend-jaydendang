use crate::base::TimestampIden;

use super::{CommonId, DMC};
use jd_utils::time::now_utc;
use modql::field::{SeaField, SeaFields};

pub fn prepare_fields_for_create<MC>(fields: &mut SeaFields, user_id: i64)
where
  MC: DMC,
{
  if MC::has_owner_id() {
    fields.push(SeaField::new(CommonId::OwnerId, user_id));
  }
  if MC::has_timestamps() {
    let now = now_utc();
    fields.push(SeaField::new(TimestampIden::Cid, user_id));
    fields.push(SeaField::new(TimestampIden::Ctime, now));
    fields.push(SeaField::new(TimestampIden::Mid, user_id));
    fields.push(SeaField::new(TimestampIden::Mtime, now));
  }
}

pub fn prepare_fields_for_update<MC>(fields: &mut SeaFields, user_id: i64)
where
  MC: DMC,
{
  if MC::has_timestamps() {
    let now = now_utc();
    fields.push(SeaField::new(TimestampIden::Mid, user_id));
    fields.push(SeaField::new(TimestampIden::Mtime, now));
  }
}
